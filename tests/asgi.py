#!/usr/bin/env python3.13
# coding:utf-8
# Copyright (C) 2025 All rights reserved.
# FILENAME:    ~~/tests/asgi.py
# VERSION:     0.0.1
# CREATED:     2025-09-01 22:49
# AUTHOR:      Sitt Guruvanich <aekasitt.g@siamintech.co.th>
# DESCRIPTION:
#
# HISTORY:
# *************************************************************

### Third-party packages ###
from fastapi.testclient import TestClient
from pytest import fixture

### Local modules ###
from examples.fastapi_endur import app


@fixture
def client():
  with TestClient(app) as client:
    yield client


def test_generate_invoice(client):
  """Test invoice generation endpoint"""
  response = client.post(
    "/invoice", json={"amount_sats": 1000, "description": "Test payment"}
  )
  assert response.status_code == 200
  result = response.json()
  assert "invoice" in result
  invoice = result["invoice"]
  assert isinstance(invoice, str)
  assert invoice.startswith("lnbc")


def test_invalid_invoice_request(client):
  """Test invoice generation with invalid parameters"""
  # Test negative amount
  response = client.post(
    "/invoice", json={"amount_sats": -1000, "description": "Test payment"}
  )
  assert response.status_code == 400

  # Test missing amount
  response = client.post("/invoice", json={"description": "Test payment"})
  assert response.status_code == 422


def test_node_status(client):
  """Test node status endpoint"""
  response = client.get("/")
  assert response.status_code == 200
  data = response.json()
  assert "status" in data
  assert data["status"] == "running"


def test_node_info(client):
  """Test node info endpoint"""
  response = client.get("/")
  assert response.status_code == 200
  data = response.json()
  assert "node_id" in data
  assert isinstance(data["node_id"], str)
