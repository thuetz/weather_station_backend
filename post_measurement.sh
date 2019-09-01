#!/bin/bash
curl -v -d '{"temperature":27.05,"humidity":37.95,"pressure":1011.72,"raw_voltage":811.00,"sensor":"DEADBEEF"}' -H "Content-Type: application/json" -X POST http://localhost:8000/v1/sensor/measurement
