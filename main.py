#!/usr/bin/env python3
from contextlib import asynccontextmanager
from endur import Endur
from fastapi import FastAPI
from fastapi.exceptions import HTTPException
from logging import Logger, getLogger
from pydantic import BaseModel

logger: Logger = getLogger("uvicorn")


@asynccontextmanager
async def lifespan(app: FastAPI):
  logger.info("Starting LDK Node...")
  app.state.node = Endur()
  try:
    node_id = app.state.node.start(data_dir="./data")
    logger.info(f"LDK Node started successfully: {node_id}")
  except Exception as e:
    logger.error(f"Failed to start LDK Node: {e}")
    raise

  yield

  logger.info("Stopping LDK Node...")
  try:
    app.state.node.stop()
    logger.info("LDK Node stopped successfully")
  except Exception as e:
    logger.error(f"Error stopping LDK Node: {e}")


app = FastAPI(lifespan=lifespan)


class InvoiceRequest(BaseModel):
  amount_sats: int
  description: str = "Payment"


@app.get("/")
async def root():
  """Get node status and basic info"""
  if not app.state.node:
    raise HTTPException(status_code=503, detail="Node not initialized")
  try:
    is_running = app.state.node.is_running()
    node_id = app.state.node.node_id() if is_running else None
    onchain_sats, lightning_sats = app.state.node.get_balances() if is_running else (0, 0)
    return {
      "status": "running" if is_running else "stopped",
      "node_id": node_id,
      "balances": {"onchain_sats": onchain_sats, "lightning_sats": lightning_sats},
    }
  except Exception as e:
    raise HTTPException(status_code=500, detail=str(e))


@app.post("/invoice")
async def create_invoice(request: InvoiceRequest):
  """Generate a Lightning invoice"""
  if not app.state.node or not app.state.node.is_running():
    raise HTTPException(status_code=503, detail="Node not running")
  try:
    invoice = app.state.node.generate_invoice(request.amount_sats, request.description)
    return {"invoice": invoice}
  except Exception as e:
    raise HTTPException(status_code=500, detail=str(e))


@app.get("/address")
async def get_address():
  """Get a new on-chain Bitcoin address"""
  if not app.state.node or not app.state.node.is_running():
    raise HTTPException(status_code=503, detail="Node not running")
  try:
    address = app.state.node.get_new_address()
    return {"address": address}
  except Exception as e:
    raise HTTPException(status_code=500, detail=str(e))


@app.get("/events")
async def get_events():
  """Process and return recent node events"""
  if not app.state.node or not app.state.node.is_running():
    raise HTTPException(status_code=503, detail="Node not running")
  try:
    events = app.state.node.process_events()
    return {"events": events}
  except Exception as e:
    raise HTTPException(status_code=500, detail=str(e))


@app.get("/balances")
async def get_balances():
  """Get node balances"""
  if not app.state.node or not app.state.node.is_running():
    raise HTTPException(status_code=503, detail="Node not running")
  try:
    onchain_sats, lightning_sats = app.state.node.get_balances()
    return {
      "onchain_sats": onchain_sats,
      "lightning_sats": lightning_sats,
      "total_sats": onchain_sats + lightning_sats,
    }
  except Exception as e:
    raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
  import uvicorn

  uvicorn.run(app, host="0.0.0.0", port=8000)
