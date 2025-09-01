#!/usr/bin/env python3.13
# coding:utf-8
# Copyright (C) 2025 All rights reserved.
# FILENAME:    ~~/tests/core.py
# VERSION:     0.0.1
# CREATED:     2025-09-01 22:49
# AUTHOR:      Sitt Guruvanich <aekasitt.g@siamintech.co.th>
# DESCRIPTION:
#
# HISTORY:
# *************************************************************

### Standard packages ###
from os import path

### Third-party packages ###
from pytest import fixture, raises
from tempfile import TemporaryDirectory

### Local modules ###
from endur import Endur


@fixture
def temp_data_dir():
  """Create a temporary directory for test data"""
  with TemporaryDirectory() as tmpdirname:
    yield tmpdirname


@fixture
def endur_node(temp_data_dir):
  """Create and start an Endur node instance"""
  node = Endur()
  try:
    node.start(data_dir=temp_data_dir)
    yield node
  finally:
    if node.is_running():
      node.stop()


def test_node_lifecycle(temp_data_dir):
  """Test basic node operations: start, status check, and stop"""
  node = Endur()

  # Test initial state
  assert not node.is_running()

  # Test start
  node_id = node.start(data_dir=temp_data_dir)
  assert isinstance(node_id, str)
  assert len(node_id) > 0
  assert node.is_running()

  # Test node_id matches
  assert node.node_id() == node_id

  # Test stop
  node.stop()
  assert not node.is_running()

  # Test restart
  node_id_2 = node.start(data_dir=temp_data_dir)
  assert node.is_running()
  assert node_id == node_id_2  # Node ID should be consistent
  node.stop()


def test_invoice_generation(endur_node):
  """Test invoice generation functionality"""
  # Test valid invoice generation
  amount_sats = 1000
  description = "Test payment"
  invoice = endur_node.generate_invoice(amount_sats, description)
  assert isinstance(invoice, str)
  assert invoice.startswith("lnbc")  # Lightning invoice prefix

  # Test invalid amount
  with raises(Exception):
    endur_node.generate_invoice(-1, description)


def test_error_handling(temp_data_dir):
  """Test error handling scenarios"""
  node = Endur()

  # Test operations on stopped node
  with raises(Exception):
    node.node_id()

  with raises(Exception):
    node.generate_invoice(1000, "test")

  # Test invalid data directory
  with raises(Exception):
    node.start(data_dir="/nonexistent/path")


def test_audit_logging(temp_data_dir):
  """Test audit log creation and content"""
  node = Endur()
  node.start(data_dir=temp_data_dir)

  # Check audit log file exists
  audit_log_path = path.join(temp_data_dir, "audit_log.txt")
  assert path.exists(audit_log_path)

  # Check basic audit log content
  with open(audit_log_path, "r") as f:
    content = f.read()
    assert "NODE_STARTED" in content

  node.stop()

  # Check stop event is logged
  with open(audit_log_path, "r") as f:
    content = f.read()
    assert "NODE_STOPPED" in content


def test_stable_channel_initialization(endur_node):
  """Test stable channel initialization and basic properties"""
  # This test assumes the stable channel is initialized with default $100 value
  # You would need to add methods to access stable channel properties for proper testing
  assert endur_node.is_running()
  # Add more specific stable channel tests based on available APIs
