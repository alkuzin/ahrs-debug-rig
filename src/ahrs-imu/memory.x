/* SPDX-License-Identifier: Apache-2.0. */
/* Copyright (C) 2026-present ahrs-debug-rig project and contributors. */

/* Memory layout for STM32F401CCU6 */
MEMORY
{
  RAM   (xrw) : ORIGIN = 0x20000000, LENGTH = 64K
  FLASH (rx)  : ORIGIN = 0x8000000,  LENGTH = 256K
}
