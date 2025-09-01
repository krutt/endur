/* ~~/src/lib.rs */

mod audit;
mod oracles;
mod stable;
mod types;

use audit::{audit_event, set_audit_log_path};
use ldk_node::{Builder, Event, Node};
use oracles::get_cached_price;
use pyo3::prelude::*;
use serde_json::json;
use std::sync::{Arc, Mutex};
use types::{Bitcoin, StableChannel, USD};

#[pyclass]
pub struct Endur {
  node: Option<Arc<Node>>,
  stable_channel: Option<Arc<Mutex<StableChannel>>>,
  audit_log_path: String,
}

#[pymethods]
impl Endur {
  #[new]
  fn new() -> Self {
    Self {
      node: None,
      stable_channel: None,
      audit_log_path: String::new(),
    }
  }

  #[pyo3(signature = (data_dir=None))]
  fn start(&mut self, data_dir: Option<String>) -> PyResult<String> {
    let mut builder = Builder::new();

    // Basic configuration
    builder.set_network(ldk_node::bitcoin::Network::Bitcoin);
    builder.set_chain_source_esplora("https://blockstream.info/api/".to_string(), None);

    let data_dir_path = data_dir.unwrap_or_else(|| "./data".to_string());
    builder.set_storage_dir_path(data_dir_path.clone());

    // Set up audit logging
    self.audit_log_path = format!("{}/audit_log.txt", data_dir_path);
    set_audit_log_path(&self.audit_log_path);

    let node = Arc::new(
      builder
        .build()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Build failed: {}", e)))?,
    );

    node
      .start()
      .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Start failed: {}", e)))?;

    let node_id = node.node_id().to_string();

    // Initialize stable channel
    let btc_price = get_cached_price();
    let stable_channel = StableChannel {
      expected_usd: USD::from_f64(100.0), // Default $100 stable value
      expected_btc: Bitcoin::from_usd(USD::from_f64(100.0), btc_price),
      latest_price: btc_price,
      ..Default::default()
    };

    self.node = Some(node);
    self.stable_channel = Some(Arc::new(Mutex::new(stable_channel)));

    audit_event(
      "NODE_STARTED",
      json!({
        "node_id": node_id,
        "btc_price": btc_price
      }),
    );

    Ok(node_id)
  }

  fn stop(&mut self) -> PyResult<()> {
    if let Some(node) = self.node.take() {
      audit_event("NODE_STOPPING", json!({}));
      node
        .stop()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Stop failed: {}", e)))?;
      audit_event("NODE_STOPPED", json!({}));
    }
    self.stable_channel = None;
    Ok(())
  }

  fn is_running(&self) -> bool {
    self.node.is_some()
  }

  fn node_id(&self) -> PyResult<String> {
    match &self.node {
      Some(node) => Ok(node.node_id().to_string()),
      None => Err(pyo3::exceptions::PyRuntimeError::new_err(
        "Node not started",
      )),
    }
  }

  fn generate_invoice(&self, amount_sats: u64, description: &str) -> PyResult<String> {
    match &self.node {
      Some(node) => {
        let msats = amount_sats * 1000;
        let desc =
          ldk_node::lightning_invoice::Description::new(description.to_string()).map_err(|e| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid description: {}", e))
          })?;
        let invoice = node
          .bolt11_payment()
          .receive(
            msats,
            &ldk_node::lightning_invoice::Bolt11InvoiceDescription::Direct(desc),
            3600,
          )
          .map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Invoice generation failed: {}", e))
          })?;
        Ok(invoice.to_string())
      }
      None => Err(pyo3::exceptions::PyRuntimeError::new_err(
        "Node not started",
      )),
    }
  }

  fn get_new_address(&self) -> PyResult<String> {
    match &self.node {
      Some(node) => {
        let address = node.onchain_payment().new_address().map_err(|e| {
          pyo3::exceptions::PyRuntimeError::new_err(format!("Address generation failed: {}", e))
        })?;
        Ok(address.to_string())
      }
      None => Err(pyo3::exceptions::PyRuntimeError::new_err(
        "Node not started",
      )),
    }
  }

  fn get_balances(&self) -> PyResult<(u64, u64)> {
    match &self.node {
      Some(node) => {
        let balances = node.list_balances();
        Ok((
          balances.total_onchain_balance_sats,
          balances.total_lightning_balance_sats,
        ))
      }
      None => Err(pyo3::exceptions::PyRuntimeError::new_err(
        "Node not started",
      )),
    }
  }

  fn process_events(&self) -> PyResult<Vec<String>> {
    match &self.node {
      Some(node) => {
        let mut events = Vec::new();
        while let Some(event) = node.next_event() {
          let event_str = match event {
            Event::ChannelReady { channel_id, .. } => {
              format!("Channel ready: {}", channel_id)
            }
            Event::PaymentReceived { amount_msat, .. } => {
              format!("Payment received: {} msats", amount_msat)
            }
            Event::PaymentSuccessful { payment_hash, .. } => {
              format!("Payment successful: {}", payment_hash)
            }
            _ => format!("Other event: {:?}", event),
          };
          events.push(event_str);
          let _ = node.event_handled();
        }
        Ok(events)
      }
      None => Err(pyo3::exceptions::PyRuntimeError::new_err(
        "Node not started",
      )),
    }
  }

  fn get_btc_price(&self) -> f64 {
    get_cached_price()
  }

  fn update_btc_price(&self) -> PyResult<f64> {
    match oracles::get_latest_price(&ureq::Agent::new()) {
      Ok(price) => {
        if let Some(stable_channel) = &self.stable_channel {
          if let Ok(mut sc) = stable_channel.lock() {
            sc.latest_price = price;
          }
        }
        audit_event("PRICE_UPDATED", json!({"btc_price": price}));
        Ok(price)
      }
      Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
        "Price fetch failed: {}",
        e
      ))),
    }
  }

  fn get_stable_channel_info(&self) -> PyResult<(f64, f64, f64, f64)> {
    match (&self.node, &self.stable_channel) {
      (Some(node), Some(stable_channel)) => {
        if let Ok(mut sc) = stable_channel.lock() {
          stable::update_balances(node, &mut sc);
          Ok((
            sc.stable_receiver_usd.0,
            sc.stable_provider_usd.0,
            sc.stable_receiver_btc.to_btc(),
            sc.stable_provider_btc.to_btc(),
          ))
        } else {
          Err(pyo3::exceptions::PyRuntimeError::new_err(
            "Failed to lock stable channel",
          ))
        }
      }
      _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
        "Node or stable channel not initialized",
      )),
    }
  }

  fn update_stability(&self) -> PyResult<()> {
    match (&self.node, &self.stable_channel) {
      (Some(node), Some(stable_channel)) => {
        let price = get_cached_price();
        if price <= 0.0 {
          return Err(pyo3::exceptions::PyRuntimeError::new_err("Invalid price"));
        }

        if let Ok(mut sc) = stable_channel.lock() {
          stable::check_stability(node, &mut sc, price);
          Ok(())
        } else {
          Err(pyo3::exceptions::PyRuntimeError::new_err(
            "Failed to lock stable channel",
          ))
        }
      }
      _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
        "Node or stable channel not initialized",
      )),
    }
  }

  fn set_stable_amount(&self, usd_amount: f64) -> PyResult<()> {
    if let Some(stable_channel) = &self.stable_channel {
      if let Ok(mut sc) = stable_channel.lock() {
        sc.expected_usd = USD::from_f64(usd_amount);
        let price = get_cached_price();
        sc.expected_btc = Bitcoin::from_usd(sc.expected_usd, price);
        audit_event(
          "STABLE_AMOUNT_SET",
          json!({
            "usd_amount": usd_amount,
            "btc_price": price
          }),
        );
        Ok(())
      } else {
        Err(pyo3::exceptions::PyRuntimeError::new_err(
          "Failed to lock stable channel",
        ))
      }
    } else {
      Err(pyo3::exceptions::PyRuntimeError::new_err(
        "Stable channel not initialized",
      ))
    }
  }
}

#[pymodule]
fn endur(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<Endur>()?;
  Ok(())
}
