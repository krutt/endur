#!/usr/bin/env python3
# coding:utf-8
# Copyright (C) 2025 All rights reserved.
# FILENAME:    ~~/examples/fastapi_endur.py
# VERSION:     0.0.1
# CREATED:     2025-08-31 22:31
# AUTHOR:      Sitt Guruvanich <aekasitt.g@siamintech.co.th>
# DESCRIPTION:
#
# HISTORY:
# *************************************************************
from contextlib import asynccontextmanager
from endur import Endur
from fastapi import FastAPI
from fastapi.exceptions import HTTPException
from logging import Logger, getLogger
from pydantic import BaseModel

logger: Logger = getLogger("uvicorn")


@asynccontextmanager
async def lifespan(app: FastAPI):
  logger.info("Starting Endur...")
  app.state.node = Endur()
  try:
    node_id = app.state.node.start(data_dir="./data")
    logger.info(f"Endur started successfully: {node_id}")
  except Exception as e:
    logger.error(f"Failed to start Endur: {e}")
    raise

  yield

  logger.info("Stopping Endur...")
  try:
    app.state.node.stop()
    logger.info("Endur stopped successfully")
  except Exception as e:
    logger.error(f"Error stopping Endur: {e}")


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


@app.get("/price")
async def get_btc_price():
   """Get current BTC/USD price"""
   if not app.state.node:
     raise HTTPException(status_code=503, detail="Node not initialized")
   try:
     price = app.state.node.get_btc_price()
     return {"btc_usd": price}
   except Exception as e:
     raise HTTPException(status_code=500, detail=str(e))


@app.post("/price/update")
async def update_btc_price():
   """Fetch latest BTC/USD price"""
   if not app.state.node or not app.state.node.is_running():
     raise HTTPException(status_code=503, detail="Node not running")
   try:
     price = app.state.node.update_btc_price()
     return {"btc_usd": price, "updated": True}
   except Exception as e:
     raise HTTPException(status_code=500, detail=str(e))


@app.get("/stable")
async def get_stable_info():
   """Get stable channel information"""
   if not app.state.node or not app.state.node.is_running():
     raise HTTPException(status_code=503, detail="Node not running")
   try:
     receiver_usd, provider_usd, receiver_btc, provider_btc = app.state.node.get_stable_channel_info()
     return {
       "stable_receiver_usd": receiver_usd,
       "stable_provider_usd": provider_usd,
       "stable_receiver_btc": receiver_btc,
       "stable_provider_btc": provider_btc,
     }
   except Exception as e:
     raise HTTPException(status_code=500, detail=str(e))


@app.post("/stable/update")
async def update_stability():
   """Update stability mechanism"""
   if not app.state.node or not app.state.node.is_running():
     raise HTTPException(status_code=503, detail="Node not running")
   try:
     app.state.node.update_stability()
     return {"status": "success", "message": "Stability updated"}
   except Exception as e:
     raise HTTPException(status_code=500, detail=str(e))


class StableAmountRequest(BaseModel):
   usd_amount: float


@app.post("/stable/amount")
async def set_stable_amount(request: StableAmountRequest):
   """Set the stable USD amount"""
   if not app.state.node or not app.state.node.is_running():
     raise HTTPException(status_code=503, detail="Node not running")
   try:
     app.state.node.set_stable_amount(request.usd_amount)
     return {"status": "success", "usd_amount": request.usd_amount}
   except Exception as e:
     raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
  import uvicorn

  uvicorn.run(app, host="0.0.0.0", port=8000)
