/* ~~/src/lib.rs */

use ldk_node::{Builder, Event, Node};
use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass]
pub struct Endur {
  node: Option<Arc<Node>>,
}

#[pymethods]
impl Endur {
  #[new]
  fn new() -> Self {
    Self { node: None }
  }

  fn start(&mut self, data_dir: Option<String>) -> PyResult<String> {
    let mut builder = Builder::new();

    // Basic configuration
    builder.set_network(ldk_node::bitcoin::Network::Bitcoin);
    builder.set_chain_source_esplora("https://blockstream.info/api/".to_string(), None);

    if let Some(dir) = data_dir {
      builder.set_storage_dir_path(dir);
    }

    let node = Arc::new(
      builder
        .build()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Build failed: {}", e)))?,
    );

    node
      .start()
      .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Start failed: {}", e)))?;

    let node_id = node.node_id().to_string();
    self.node = Some(node);

    Ok(node_id)
  }

  fn stop(&mut self) -> PyResult<()> {
    if let Some(node) = self.node.take() {
      node
        .stop()
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Stop failed: {}", e)))?;
    }
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
}

#[pymodule]
fn endur(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<Endur>()?;
  Ok(())
}

