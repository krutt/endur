/* ~~/src/stable.rs */

use crate::audit::audit_event;
use crate::oracles::get_cached_price;
use crate::types::{Bitcoin, StableChannel, USD};
use ldk_node::{lightning::ln::types::ChannelId, Node};
use serde_json::json;
use ureq::Agent;

/// Get the current BTC/USD price, preferring cached value when available
pub fn get_current_price(agent: &Agent) -> f64 {
  // First try the cached price
  let cached_price = get_cached_price();

  // Use the cached price if valid
  if cached_price > 0.0 {
    return cached_price;
  }

  match crate::oracles::get_latest_price(agent) {
    Ok(price) => price,
    Err(_) => 0.0,
  }
}

pub fn update_balances<'update_balance_lifetime>(
  node: &Node,
  stable_channel: &'update_balance_lifetime mut StableChannel,
) -> (bool, &'update_balance_lifetime mut StableChannel) {
  if stable_channel.latest_price == 0.0 {
    stable_channel.latest_price = get_cached_price();

    if stable_channel.latest_price == 0.0 {
      let agent = Agent::new();
      stable_channel.latest_price = get_current_price(&agent);
    }
  }

  // --- Update On-chain ---
  let balances = node.list_balances();
  stable_channel.onchain_btc = Bitcoin::from_sats(balances.total_onchain_balance_sats);
  stable_channel.onchain_usd = USD::from_bitcoin(stable_channel.onchain_btc, stable_channel.latest_price);

  let channels = node.list_channels();
  let matching_channel = if stable_channel.channel_id == ChannelId::from_bytes([0; 32]) {
    channels.first()
  } else {
    channels.iter().find(|c| c.channel_id == stable_channel.channel_id)
  };

  if let Some(channel) = matching_channel {
    if stable_channel.channel_id == ChannelId::from_bytes([0; 32]) {
      stable_channel.channel_id = channel.channel_id;
      println!("Set active channel ID to: {}", stable_channel.channel_id);
    }

    let unspendable_punishment_sats = channel.unspendable_punishment_reserve.unwrap_or(0);
    let our_balance_sats = (channel.outbound_capacity_msat / 1000) + unspendable_punishment_sats;
    let their_balance_sats = channel.channel_value_sats - our_balance_sats;

    if stable_channel.is_receiver {
      stable_channel.receiver_btc = Bitcoin::from_sats(our_balance_sats);
      stable_channel.provider_btc = Bitcoin::from_sats(their_balance_sats);
    } else {
      stable_channel.provider_btc = Bitcoin::from_sats(our_balance_sats);
      stable_channel.receiver_btc = Bitcoin::from_sats(their_balance_sats);
    }

    stable_channel.receiver_usd = USD::from_bitcoin(stable_channel.receiver_btc, stable_channel.latest_price);
    stable_channel.provider_usd = USD::from_bitcoin(stable_channel.provider_btc, stable_channel.latest_price);

    audit_event(
      "BALANCE_UPDATE",
      json!({
          "channel_id": format!("{}", stable_channel.channel_id),
          "receiver_btc": stable_channel.receiver_btc.to_string(),
          "provider_btc": stable_channel.provider_btc.to_string(),
          "receiver_usd": stable_channel.receiver_usd.to_string(),
          "provider_usd": stable_channel.provider_usd.to_string(),
          "btc_price": stable_channel.latest_price
      }),
    );

    return (true, stable_channel);
  }

  println!("No matching channel found for ID: {}", stable_channel.channel_id);
  (true, stable_channel)
}

pub fn check_stability(node: &Node, stable_channel: &mut StableChannel, price: f64) {
  let current_price = if price > 0.0 {
    price
  } else {
    let cached_price = get_cached_price();
    if cached_price > 0.0 {
      cached_price
    } else {
      audit_event(
        "STABILITY_SKIP",
        json!({
            "reason": "no valid price available"
        }),
      );
      return;
    }
  };

  stable_channel.latest_price = current_price;
  let (success, _) = update_balances(node, stable_channel);
  if !success {
    audit_event(
      "BALANCE_UPDATE_FAILED",
      json!({
          "channel_id": format!("{}", stable_channel.channel_id)
      }),
    );
    return;
  }

  let dollars_from_par = stable_channel.receiver_usd - stable_channel.expected_usd;
  let percent_from_par = ((dollars_from_par / stable_channel.expected_usd) * 100.0).abs();
  let is_receiver_below_expected = stable_channel.receiver_usd < stable_channel.expected_usd;

  let action = if percent_from_par < 0.1 {
    "STABLE"
  } else if stable_channel.risk_level > 100 {
    "HIGH_RISK_NO_ACTION"
  } else if (stable_channel.is_receiver && is_receiver_below_expected)
    || (!stable_channel.is_receiver && !is_receiver_below_expected)
  {
    "CHECK_ONLY"
  } else {
    "PAY"
  };

  audit_event(
    "STABILITY_CHECK",
    json!({
      "expected_usd": stable_channel.expected_usd.0,
      "current_receiver_usd": stable_channel.receiver_usd.0,
      "percent_from_par": percent_from_par,
      "btc_price": stable_channel.latest_price,
      "action": action,
      "is_receiver": stable_channel.is_receiver,
      "risk_level": stable_channel.risk_level
    }),
  );

  if action != "PAY" {
    return;
  }

  let amt = USD::to_msats(dollars_from_par, stable_channel.latest_price);
  match node.spontaneous_payment().send(amt, stable_channel.counterparty, None) {
    Ok(payment_id) => {
      stable_channel.payment_made = true;
      audit_event(
        "STABILITY_PAYMENT_SENT",
        json!({
            "amount_msats": amt,
            "payment_id": payment_id.to_string(),
            "counterparty": stable_channel.counterparty.to_string()
        }),
      );
    }
    Err(e) => {
      audit_event(
        "STABILITY_PAYMENT_FAILED",
        json!({
            "amount_msats": amt,
            "error": format!("{e}"),
            "counterparty": stable_channel.counterparty.to_string()
        }),
      );
    }
  }
}
