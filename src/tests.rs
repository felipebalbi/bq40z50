macro_rules! bq40z50_tests {
    ($revision:ident) => {
        #[cfg(test)]
        mod tests {
            // The audit reproduction tests at the bottom of this module quote TRM section
            // titles verbatim in their doc comments (`ManufacturerAccess() 0x0054
            // OperationStatus`, `DataFlashAccess`, `SafetyStatus`, ...). clippy::doc_markdown
            // reads those as un-backticked Rust items. Backticking prose lifted from the
            // spec would make the citations harder to match against the PDF, so the lint is
            // relaxed here. Test-only; the library is unaffected.
            #![allow(clippy::doc_markdown)]

            use embedded_batteries_async::smart_battery::SmartBattery;
            use embedded_hal_mock::eh1::delay::{CheckedDelay, NoopDelay, Transaction as DelayTransaction};
            use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
            use $revision as Bq40z50;

            use super::*;
            use crate::common::{CapacityModeState, Config};
            use crate::consts::BQ_ADDR;

            // Needed to compile in the pender symbol for embassy-time.
            // This is only enabled during tests and when actually using the driver,
            // the user must provide embassy-time symbols.
            #[cfg(feature = "embassy-timeout")]
            #[allow(dead_code)]
            fn setup() -> &'static mut embassy_executor::Executor {
                static EXECUTOR: static_cell::StaticCell<embassy_executor::Executor> = static_cell::StaticCell::new();
                EXECUTOR.init(embassy_executor::Executor::new())
            }

            #[tokio::test]
            async fn update_config() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x02, 0x00, 0x46]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x0A, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0,
                        ],
                    ),
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x02, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x0A, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        ],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new_with_config(
                    i2c,
                    NoopDelay::new(),
                    Config {
                        pec_read: true,
                        ..Default::default()
                    },
                );

                bq.device.mac_firmware_version().dispatch_async().await.unwrap();

                // Change the device config to not use PEC.
                let mut config = bq.config();
                config.pec_read = false;
                bq.update_config(config);
                bq.device.mac_firmware_version().dispatch_async().await.unwrap();

                bq.device.interface.i2c.done();
            }

            #[tokio::test]
            async fn read_chip_id() {
                let expectations = vec![Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x21, 0x00])];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                bq.mac_gauging().dispatch_async().await.unwrap();

                bq.interface.i2c.done();
            }

            #[tokio::test]
            async fn read_chip_id_pec() {
                let expectations = vec![Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x21, 0x00, 0xD7])];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new_with_config(
                    i2c,
                    NoopDelay::new(),
                    Config {
                        pec_write: true,
                        ..Default::default()
                    },
                ));

                bq.mac_gauging().dispatch_async().await.unwrap();

                bq.interface.i2c.done();
            }

            #[tokio::test]
            async fn read_chip_id_2() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x01, 0x00]),
                    Transaction::write_read(BQ_ADDR, vec![0x44], vec![0x04, 0x01, 0x00, 0x00, 0x00]),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                bq.mac_device_type().dispatch_async().await.unwrap();
                bq.interface.i2c.done();
            }

            #[tokio::test]
            async fn read_firmware_version() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x02, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x0A, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        ],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                bq.mac_firmware_version().dispatch_async().await.unwrap();
                bq.interface.i2c.done();
            }

            #[tokio::test]
            async fn read_firmware_version_pec() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x02, 0x00, 0x46]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x0A, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0,
                        ],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new_with_config(
                    i2c,
                    NoopDelay::new(),
                    Config {
                        pec_read: true,
                        ..Default::default()
                    },
                ));

                bq.mac_firmware_version().dispatch_async().await.unwrap();
                bq.interface.i2c.done();
            }

            #[tokio::test]
            async fn read_too_large_manufacture_name() {
                let expectations = vec![Transaction::write_read(BQ_ADDR, vec![0x20], vec![0x00])];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                let mut manufacture_name = [0u8; 1];

                bq.manufacture_name()
                    .read_async(&mut manufacture_name)
                    .await
                    .unwrap();
                bq.interface.i2c.done();
            }

            #[tokio::test]
            async fn write_unseal_keys() {
                let expectations = vec![
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 0x0A, 0x35, 0x00, 0x30, 0x30, 0x60, 0x60, 0x01, 0x01, 0x10, 0x10,
                        ],
                    ),
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x35, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x0A, 0x35, 0x00, 0x30, 0x30, 0x60, 0x60, 0x01, 0x01, 0x10, 0x10],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                let security_keys = [0x30u8, 0x30u8, 0x60u8, 0x60u8, 0x01u8, 0x01u8, 0x10u8, 0x10u8];

                bq.write_security_keys(&security_keys).await.unwrap();

                let mut result = [0u8; 8];
                bq.read_security_keys(&mut result).await.unwrap();

                assert_eq!(u16::from_le_bytes([result[0], result[1]]), 0x3030);
                assert_eq!(u16::from_le_bytes([result[2], result[3]]), 0x6060);
                assert_eq!(u16::from_le_bytes([result[4], result[5]]), 0x0101);
                assert_eq!(u16::from_le_bytes([result[6], result[7]]), 0x1010);
                bq.device.interface.i2c.done();
            }

            #[tokio::test]
            async fn test_battery_status() {
                let expectations = vec![Transaction::write_read(BQ_ADDR, vec![0x16], vec![0x30, 0x30])];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                let status = match bq.battery_status().await {
                    Ok(status) => status,
                    Err(e) => match e {
                        _ => unreachable!(),
                    },
                };

                assert_eq!(status.error_code(), ErrorCode::Ok);

                bq.device.interface.i2c.done();
            }

            #[tokio::test]
            async fn test_battery_status_pec() {
                // Full bus transaction looks like [ 0x0B << 1 || 0x16 || 0x0B << 1 | 1 || 0x30 || 0x30 || 0xB7 (PEC)]
                let expectations = vec![Transaction::write_read(
                    BQ_ADDR,
                    vec![0x16],
                    vec![0x30, 0x30, 0xB7],
                )];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new_with_config(
                    i2c,
                    NoopDelay::new(),
                    Config {
                        pec_read: true,
                        ..Default::default()
                    },
                );

                let status = match bq.battery_status().await {
                    Ok(status) => status,
                    Err(e) => match e {
                        _ => unreachable!(),
                    },
                };

                assert_eq!(status.error_code(), ErrorCode::Ok);

                bq.device.interface.i2c.done();
            }

            #[tokio::test]
            #[allow(unsafe_code)]
            async fn test_read_write_unchecked() {
                let expectations = vec![
                    Transaction::write_read(BQ_ADDR, vec![0x16], vec![0x30, 0x30]),
                    Transaction::write(BQ_ADDR, vec![0x16, 0x2F, 0x30]),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                let mut data = [0u8; 2];

                unsafe {
                    bq.read_register_unchecked(0x16, &mut data).await.unwrap();
                }
                data[0] -= 1;

                unsafe {
                    bq.write_register_unchecked(0x16, &data).await.unwrap();
                }

                bq.device.interface.i2c.done();
            }

            #[tokio::test]
            async fn test_capacity_mode() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x03, 0x00, 0x80]),
                    Transaction::write_read(BQ_ADDR, vec![0x0F], vec![100, 0x00]),
                    Transaction::write(BQ_ADDR, vec![0x03, 0x00, 0x00]),
                    Transaction::write_read(BQ_ADDR, vec![0x0F], vec![80, 0x00]),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                assert_eq!(bq.capacity_mode_state.get(), CapacityModeState::Milliamps);

                let mode = BatteryModeFields::new().with_capacity_mode(true);
                bq.set_battery_mode(mode).await.unwrap();
                assert_eq!(bq.capacity_mode_state.get(), CapacityModeState::Centiwatt);

                let mode = BatteryModeFields::new().with_capacity_mode(false);
                let rem_cap = bq.remaining_capacity().await.unwrap();
                assert!(matches!(rem_cap, CapacityModeValue::CentiWattUnsigned(100)));

                let _info = bq.set_battery_mode(mode).await;
                assert_eq!(bq.capacity_mode_state.get(), CapacityModeState::Milliamps);

                let rem_cap = bq.remaining_capacity().await.unwrap();
                assert!(matches!(rem_cap, CapacityModeValue::MilliAmpUnsigned(80)));

                bq.device.interface.i2c.done();
            }

            #[tokio::test]
            async fn test_reg_retries() {
                // Should have 3 retries
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x17, 100, 0]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    ),
                    Transaction::write(BQ_ADDR, vec![0x17, 100, 0]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    ),
                    Transaction::write(BQ_ADDR, vec![0x17, 100, 0]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    ),
                    Transaction::write(BQ_ADDR, vec![0x17, 100, 0]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let delay_expectations = vec![
                    DelayTransaction::delay_ms(10),
                    DelayTransaction::delay_ms(10),
                    DelayTransaction::delay_ms(10),
                ];
                let mut bq = Bq40z50::new(i2c, CheckedDelay::new(&delay_expectations));

                let res = bq
                    .device
                    .cycle_count()
                    .write_async(|f| f.set_cycle_count(100))
                    .await;

                assert_eq!(
                    res,
                    Err(BQ40Z50Error::I2c(embedded_hal::i2c::ErrorKind::NoAcknowledge(
                        embedded_hal::i2c::NoAcknowledgeSource::Address
                    )))
                );

                bq.device.interface.i2c.done();
                bq.device.interface.delay.done();
            }

            #[tokio::test]
            async fn test_cmd_retries() {
                // Should have 3 retries
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 2, 0x53, 0]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    ),
                    Transaction::write(BQ_ADDR, vec![0x44, 2, 0x53, 0]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    ),
                    Transaction::write(BQ_ADDR, vec![0x44, 2, 0x53, 0]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    ),
                    Transaction::write(BQ_ADDR, vec![0x44, 2, 0x53, 0]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let delay_expectations = vec![
                    DelayTransaction::delay_ms(10),
                    DelayTransaction::delay_ms(10),
                    DelayTransaction::delay_ms(10),
                ];
                let mut bq = Bq40z50::new(i2c, CheckedDelay::new(&delay_expectations));

                let res = bq.device.mac_pf_status().dispatch_async().await;

                assert_eq!(
                    res,
                    Err(BQ40Z50Error::I2c(embedded_hal::i2c::ErrorKind::NoAcknowledge(
                        embedded_hal::i2c::NoAcknowledgeSource::Address
                    )))
                );

                bq.device.interface.i2c.done();
                bq.device.interface.delay.done();
            }

            #[cfg(not(feature = "r1"))]
            #[tokio::test]
            async fn test_charging_override_voltage() {
                let expectations = vec![
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 0x0C, 0xB0, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E,
                        ],
                    ),
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0xB0, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x0C, 0xB0, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E,
                        ],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                bq.write_charging_voltage_override(&ChargingVoltageOverride {
                    low_temp_chrg_mv: 11800,
                    std_low_temp_chrg_mv: 12000,
                    std_hi_temp_chrg_mv: 12600,
                    hi_temp_chrg_mv: 12000,
                    recommended_temp_chrg_mv: 11800,
                })
                .await
                .unwrap();

                let override_struct = bq.read_charging_voltage_override().await.unwrap();

                assert_eq!(override_struct.low_temp_chrg_mv, 11800);
                assert_eq!(override_struct.std_low_temp_chrg_mv, 12000);
                assert_eq!(override_struct.std_hi_temp_chrg_mv, 12600);
                assert_eq!(override_struct.hi_temp_chrg_mv, 12000);
                assert_eq!(override_struct.recommended_temp_chrg_mv, 11800);
                bq.device.interface.i2c.done();
            }

            #[cfg(not(any(feature = "r1", feature = "r3")))]
            #[tokio::test]
            async fn test_read_mfg_info_c() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x7B, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x7B, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x18, 0x2E,
                            0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                            0x2E, 0x18, 0x2E, 0x44, 0x32,
                        ],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new_with_config(i2c, NoopDelay::new(), Config { ..Default::default() });

                let mut buf = [0u8; 32];
                bq.read_mfg_info_c(&mut buf).await.unwrap();

                bq.device.interface.i2c.done();
            }

            #[cfg(not(any(feature = "r1", feature = "r3")))]
            #[tokio::test]
            async fn test_read_mfg_info_c_pec() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x7B, 0x00, 89]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x7B, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x18, 0x2E,
                            0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                            0x2E, 0x18, 0x2E, 0x44, 0x32, 0xD4,
                        ],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new_with_config(
                    i2c,
                    NoopDelay::new(),
                    Config {
                        pec_read: true,
                        pec_write: true,
                        ..Default::default()
                    },
                );

                let mut buf = [0u8; 32];
                bq.read_mfg_info_c(&mut buf).await.unwrap();

                bq.device.interface.i2c.done();
            }

            #[tokio::test]
            async fn test_df_transactions() {
                let expectations = vec![
                    // Write 1, 4 bytes (1 block write)
                    Transaction::write(BQ_ADDR, vec![0x44, 0x06, 0x00, 0x40, 0xFE, 0xCA, 0xFE, 0xC0]),
                    // Write 2, 48 bytes (2 block writes)
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x00, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                        ],
                    ),
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 18, 0x20, 0x40, 0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE,
                            0xC0, 0xFE, 0xCA, 0xFE, 0xC0,
                        ],
                    ),
                    // Write 3, 128 bytes (4 block writes)
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x00, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                        ],
                    ),
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x20, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                        ],
                    ),
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x40, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                        ],
                    ),
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x60, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                        ],
                    ),
                    // Read 1, 4 bytes (1 block read)
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x00, 0x40]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x22, 0x00, 0x40, 0x01, 0x02, 0x03, 0x04],
                    ),
                    // Read 2, 48 bytes (2 block reads)
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x00, 0x40]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18,
                        ],
                    ),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x20, 0x40, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD,
                            0xDE, 0xAD, 0xD0, 0x0D,
                        ],
                    ),
                    // Read 3, 128 bytes (4 block reads)
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x00, 0x40]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18,
                        ],
                    ),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18,
                        ],
                    ),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18,
                        ],
                    ),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18,
                        ],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                // Write 1
                let write = [0xFEu8, 0xCA, 0xFE, 0xC0];
                bq.write_dataflash(0x4000, &write).await.unwrap();

                // Write 2
                let write = [
                    0x03u8, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                    0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE, 0xC0,
                ];
                bq.write_dataflash(0x4000, &write).await.unwrap();

                // Write 3
                let write = [
                    0x03u8, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                    0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                    0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                    0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                ];
                bq.write_dataflash(0x4000, &write).await.unwrap();

                // Read 1
                let mut read = [0u8; 48];
                bq.read_dataflash(0x4000, &mut read[..4]).await.unwrap();
                assert_eq!(read[..4], [0x01, 0x02, 0x03, 0x04]);

                // Read 2
                bq.read_dataflash(0x4000, &mut read).await.unwrap();
                assert_eq!(
                    read,
                    [
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18,
                        0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xD0, 0x0D
                    ]
                );

                // Read 3
                let mut read = [0u8; 128];
                bq.read_dataflash(0x4000, &mut read).await.unwrap();

                assert_eq!(
                    read,
                    [
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18,
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18,
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18,
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18
                    ]
                );

                bq.device.interface.i2c.done();
            }

            #[tokio::test]
            async fn test_df_transactions_pec() {
                let expectations = vec![
                    // Write 1, 4 bytes (1 block write)
                    Transaction::write(
                        BQ_ADDR,
                        vec![0x44, 0x06, 0x00, 0x40, 0xFE, 0xCA, 0xFE, 0xC0, 0x41],
                    ),
                    // Write 2, 48 bytes (2 block writes)
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x00, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0xA2, // PEC
                        ],
                    ),
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 18, 0x20, 0x40, 0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE,
                            0xC0, 0xFE, 0xCA, 0xFE, 0xC0, 0x32, // PEC
                        ],
                    ),
                    // Write 3, 128 bytes (4 block writes)
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x00, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0xA2, // PEC
                        ],
                    ),
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x20, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x14, // PEC
                        ],
                    ),
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x40, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0xC9, // PEC
                        ],
                    ),
                    Transaction::write(
                        BQ_ADDR,
                        vec![
                            0x44, 34, 0x60, 0x40, 0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E,
                            0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18,
                            0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x7F, // PEC
                        ],
                    ),
                    // PEC reads will always read in 32 byte data chunks.
                    // Read 1, 4 bytes (1 block read)
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x00, 0x40, 0xAB /* PEC */]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04,
                            0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03,
                            0x04, 0x01, 0x02, 0x03, 0x04, 0x44, /* PEC */
                        ],
                    ),
                    // Read 2, 48 bytes (2 block reads)
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x00, 0x40, 0xAB /* PEC */]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18, 0x22, // PEC
                        ],
                    ),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x20, 0x40, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD,
                            0xDE, 0xAD, 0xD0, 0x0D, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE,
                            0xAD, 0xDE, 0xAD, 0xD0, 0x0D, 0xE0, // PEC
                        ],
                    ),
                    // Read 3, 128 bytes (4 block reads)
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x00, 0x40, 0xAB /* PEC */]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18, 0x22, // PEC
                        ],
                    ),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18, 0x22, // PEC
                        ],
                    ),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18, 0x22, // PEC
                        ],
                    ),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![
                            0x22, 0x00, 0x40, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00,
                            0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                            0x38, 0x31, 0xE0, 0x2E, 0x18, 0x22, // PEC
                        ],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new_with_config(
                    i2c,
                    NoopDelay::new(),
                    Config {
                        pec_read: true,
                        pec_write: true,
                        ..Default::default()
                    },
                );

                // Write 1
                let write = [0xFEu8, 0xCA, 0xFE, 0xC0];
                bq.write_dataflash(0x4000, &write).await.unwrap();

                // Write 2
                let write = [
                    0x03u8, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                    0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE, 0xC0, 0xFE, 0xCA, 0xFE, 0xC0,
                ];
                bq.write_dataflash(0x4000, &write).await.unwrap();

                // Write 3
                let write = [
                    0x03u8, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                    0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                    0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                    0x03, 0x56, 0x01, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E,
                    0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0,
                ];
                bq.write_dataflash(0x4000, &write).await.unwrap();

                // Read 1
                let mut read = [0u8; 48];
                bq.read_dataflash(0x4000, &mut read[..4]).await.unwrap();
                assert_eq!(read[..4], [0x01, 0x02, 0x03, 0x04]);

                // Read 2
                bq.read_dataflash(0x4000, &mut read).await.unwrap();
                assert_eq!(
                    read,
                    [
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18,
                        0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xDE, 0xAD, 0xD0, 0x0D
                    ]
                );

                // Read 3
                let mut read = [0u8; 128];
                bq.read_dataflash(0x4000, &mut read).await.unwrap();

                assert_eq!(
                    read,
                    [
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18,
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18,
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18,
                        0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E,
                        0x38, 0x31, 0xE0, 0x2E, 0x18, 0x2E, 0x00, 0x18, 0x2E, 0xE0, 0x2E, 0x38, 0x31, 0xE0, 0x2E, 0x18
                    ]
                );

                bq.device.interface.i2c.done();
            }

            // ================================================================
            // TRM conformance audit -- RED baseline
            // (branch review/trm-conformance-audit, audit base 67132680)
            //
            // EVERY test below this line asserts the behaviour the Technical
            // Reference Manuals require, and therefore FAILS on the audit base
            // commit. They are a to-do list expressed as a test suite: each
            // failure is one real defect, and "cargo test" going green is the
            // definition of done.
            //
            // Nothing below is #[ignore]d. An ignored test is an invisible test.
            //
            // See REVIEW_FINDINGS.md at the repo root for the full write-up,
            // spec citations, severity ranking and the findings that could not
            // be expressed as a test at all.
            //
            // Each test names the pattern it uses to stay COMPILABLE while RED:
            //
            //   Pattern A - the accessor survives the fix, only its bit or its
            //               wire encoding moves. Test needs no edit when fixed.
            //   Pattern B - the accessor is renamed or deleted by the fix, so
            //               the new name cannot be written down yet. The test
            //               asserts that the CURRENTLY WRONG accessor does NOT
            //               report the bit. Carries a "WHEN FIXED:" line saying
            //               exactly what to replace it with.
            //   Pattern D - the fix changes a function signature. The call site
            //               uses today's signature; the assertion is already the
            //               TRM-correct one. Carries a "WHEN FIXED:" line.
            //   Pattern E - the driver panics where it should return an error.
            //               The test asserts the Err and fails by panicking.
            //
            // (Pattern C - the defect is a MISSING field, so there is no method
            // to name and no test that would compile. Those findings live in
            // REVIEW_FINDINGS.md only.)
            // ================================================================

            // ---------------- C1: protection/safety register map ----------------

            /// Finding C1 (critical): the five protection/safety blocks
            /// (SAFETY_ALERT, SAFETY_STATUS, PF_ALERT, PF_STATUS, OPERATION_STATUS)
            /// are byte-identical across device_r1.yaml, device_r3.yaml,
            /// device_r4.yaml and device_r5.yaml -- same content at the same line
            /// numbers (217, 292, 373, 448, 532). They were copied from the R1
            /// manifest and never re-derived. TI remapped several of these bits at
            /// R3, so r3/r4/r5 ship a register map belonging to different silicon.
            ///
            /// OperationStatus bit 29. SLUUCN4B (BQ40Z50-R5 TRM) §16.1.41
            /// "ManufacturerAccess() 0x0054 OperationStatus": "DISCONN (Bit 29):
            /// System disconnect". SLUUCH2 (BQ40Z50-R4 TRM) §16.1.40 agrees.
            /// The manifest calls it EMSHUT, which is the R1 name
            /// (SLUUA43A §12.1.40: "EMSHUT (Bit 29): Emergency Shutdown") -- hence
            /// the cfg gate, r1 is genuinely correct here.
            ///
            /// Consequence: a host seeing a system disconnect is told the pack is
            /// in emergency FET shutdown. Two different faults, two different
            /// recovery paths.
            ///
            /// Offending manifest entry: device_r5.yaml:624.
            ///
            /// PATTERN B. WHEN FIXED: delete this test and replace it with one
            /// asserting `status.disconn() == true` for bit 29 on r3/r4/r5.
            #[cfg(not(feature = "r1"))]
            #[tokio::test]
            async fn finding_c1_operation_status_bit29_is_disconn_not_emshut() {
                // OperationStatus = 0x2000_0000 (bit 29 only), little endian.
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x54, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x06, 0x54, 0x00, 0x00, 0x00, 0x00, 0x20],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                let status = bq.mac_operation_status().dispatch_async().await.unwrap();

                assert!(
                    !status.emshut(),
                    "TRM-correct behaviour: OperationStatus bit 29 is DISCONN (system \
                     disconnect) on R3/R4/R5, so emshut() must NOT be wired to it. \
                     SLUUCN4B 16.1.41. The driver currently reports emergency FET \
                     shutdown for a system disconnect."
                );

                bq.interface.i2c.done();
            }

            /// Finding C1 (critical), continued: OperationStatus bit 26.
            ///
            /// SLUUCN4B §16.1.41 "ManufacturerAccess() 0x0054 OperationStatus":
            /// "STORAGEM (Bit 26): Storage Mode is triggered via command".
            /// SLUUCH2 §16.1.40 types the same bit "VLB (Bit 26): Very low battery
            /// warning". Neither is SLPAD; SLPAD is the R1 name
            /// (SLUUA43A §12.1.40: "SLPAD (Bit 26): ADC Measurement in SLEEP mode").
            ///
            /// Offending manifest entry: device_r5.yaml:615.
            ///
            /// PATTERN B. WHEN FIXED: delete this test and replace it with one
            /// asserting `status.storagem() == true` for bit 26 on r5, and
            /// `status.vlb() == true` on r4.
            #[cfg(not(feature = "r1"))]
            #[tokio::test]
            async fn finding_c1_operation_status_bit26_is_not_slpad() {
                // OperationStatus = 0x0400_0000 (bit 26 only), little endian.
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x54, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x06, 0x54, 0x00, 0x00, 0x00, 0x00, 0x04],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                let status = bq.mac_operation_status().dispatch_async().await.unwrap();

                assert!(
                    !status.slpad(),
                    "TRM-correct behaviour: OperationStatus bit 26 is STORAGEM on R5 \
                     (SLUUCN4B 16.1.41) and VLB on R4 (SLUUCH2 16.1.40), so slpad() \
                     must NOT be wired to it. SLPAD is the R1 name only."
                );

                bq.interface.i2c.done();
            }

            /// Finding C1 (critical), continued: the real EMSHUT.
            ///
            /// EMSHUT does exist on R5 -- at bit 6, not bit 29. SLUUCN4B §16.1.41
            /// "ManufacturerAccess() 0x0054 OperationStatus": "EMSHUT (Bit 6):
            /// Emergency FET Shutdown". The manifest has no accessor for bit 6 at
            /// all, and spends the name on bit 29 instead (see the bit-29 test
            /// above). So emergency FET shutdown -- the most urgent thing this
            /// register reports -- is unreportable by the driver.
            ///
            /// Offending manifest entry: the whole MAC_OPERATION_STATUS block,
            /// device_r5.yaml:532 onwards, unchanged since R1.
            ///
            /// PATTERN A -- this is the ideal case: `emshut()` survives the fix,
            /// only its bit moves from 29 to 6. This test needs NO edit when fixed.
            #[cfg(not(feature = "r1"))]
            #[tokio::test]
            async fn finding_c1_operation_status_emshut_belongs_at_bit6() {
                // OperationStatus = 0x0000_0040 (bit 6 only), little endian.
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x54, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x06, 0x54, 0x00, 0x40, 0x00, 0x00, 0x00],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                let status = bq.mac_operation_status().dispatch_async().await.unwrap();

                assert!(
                    status.emshut(),
                    "TRM-correct behaviour: SLUUCN4B 16.1.41 puts EMSHUT (Emergency FET \
                     Shutdown) at bit 6. The driver wires emshut() to bit 29, so an \
                     emergency FET shutdown reads as false and the fault is invisible."
                );

                bq.interface.i2c.done();
            }

            /// Finding C1 (critical), continued: PFStatus bit 25.
            ///
            /// SLUUCN4B §16.1.40 "ManufacturerAccess() 0x0053 PFStatus": "FORCE
            /// (Bit 25): Manual PF". The manifest calls it OPNCELL, which is the R1
            /// name (SLUUA43A §12.1.39: "OPNCELL (Bit 25): Open Cell Tab Connection
            /// Failure").
            ///
            /// This is the most dangerous mislabel in the set: a permanent fail the
            /// HOST ITSELF REQUESTED is reported as an open cell-tab hardware
            /// failure, i.e. a physically broken pack.
            ///
            /// Offending manifest entry: device_r5.yaml:515.
            ///
            /// PATTERN B. WHEN FIXED: delete this test and replace it with one
            /// asserting `status.force() == true` for bit 25 on r3/r4/r5.
            #[cfg(not(feature = "r1"))]
            #[tokio::test]
            async fn finding_c1_pf_status_bit25_is_force_not_opncell() {
                // PFStatus = 0x0200_0000 (bit 25 only), little endian.
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x53, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x06, 0x53, 0x00, 0x00, 0x00, 0x00, 0x02],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                let status = bq.mac_pf_status().dispatch_async().await.unwrap();

                assert!(
                    !status.opncell(),
                    "TRM-correct behaviour: PFStatus bit 25 is FORCE (Manual PF) on \
                     R3/R4/R5 -- SLUUCN4B 16.1.40 -- so opncell() must NOT be wired to \
                     it. The driver reports a host-requested permanent fail as an open \
                     cell-tab hardware failure."
                );

                bq.interface.i2c.done();
            }

            /// Finding C1 (critical), continued: PFAlert bit 25 is Reserved.
            ///
            /// SLUUCN4B §16.1.39 "ManufacturerAccess() 0x0052 PFAlert": "RSVD (Bits
            /// 26-23): Reserved. Do not use." The manifest exposes bit 25 as OPNC,
            /// which is the R1 name (SLUUA43A §12.1.38: "OPNC (Bit 25): Open Cell
            /// Tab Connection Failure").
            ///
            /// The driver will therefore report a protection event that the silicon
            /// does not define, from a bit whose value TI does not guarantee.
            ///
            /// Offending manifest entry: device_r5.yaml:434.
            ///
            /// PATTERN B. WHEN FIXED: delete this test; bit 25 should have no
            /// accessor at all on r3/r4/r5, which is not something a test can assert.
            #[cfg(not(feature = "r1"))]
            #[tokio::test]
            async fn finding_c1_pf_alert_bit25_is_reserved() {
                // PFAlert = 0x0200_0000 (bit 25 only), little endian.
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x52, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x06, 0x52, 0x00, 0x00, 0x00, 0x00, 0x02],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                let alert = bq.mac_pf_alert().dispatch_async().await.unwrap();

                assert!(
                    !alert.opnc(),
                    "TRM-correct behaviour: PFAlert bit 25 falls inside \"RSVD (Bits \
                     26-23): Reserved. Do not use.\" on R3/R4/R5 -- SLUUCN4B 16.1.39 -- \
                     so there must be no opnc() reporting it."
                );

                bq.interface.i2c.done();
            }

            /// Finding C1 (critical), continued: SafetyStatus bit 19 is Reserved,
            /// but SafetyAlert bit 19 is NOT.
            ///
            /// SLUUCN4B §16.1.38 "ManufacturerAccess() 0x0051 SafetyStatus": "RSVD
            /// (Bit 19): Reserved. Do not use." The manifest exposes it as PTOS,
            /// the R1 name (SLUUA43A §12.1.36).
            ///
            /// The first assertion below is a POSITIVE CONTROL and is expected to
            /// pass both before and after the fix. SLUUCN4B §16.1.37
            /// "ManufacturerAccess() 0x0050 SafetyAlert" genuinely does define
            /// "PTOS (Bit 19): Precharge timeout suspend" (and "CTOS (Bit 21):
            /// Charge timeout suspend"), so the SAFETY_ALERT block is RIGHT about
            /// those two bits and must not be changed. Only SafetyStatus bit 19 is
            /// reserved. The control is here so that a fix for SafetyStatus cannot
            /// be over-applied to SafetyAlert without this test noticing.
            ///
            /// Offending manifest entry: device_r5.yaml:350 (SafetyStatus PTOS).
            ///
            /// PATTERN B. WHEN FIXED: keep the SafetyAlert control assertion; delete
            /// the SafetyStatus assertion, since bit 19 should then have no accessor.
            #[cfg(not(feature = "r1"))]
            #[tokio::test]
            async fn finding_c1_safety_status_bit19_is_reserved_but_safety_alert_bit19_is_ptos() {
                // Both registers = 0x0008_0000 (bit 19 only), little endian.
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x50, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x06, 0x50, 0x00, 0x00, 0x00, 0x08, 0x00],
                    ),
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x51, 0x00]),
                    Transaction::write_read(
                        BQ_ADDR,
                        vec![0x44],
                        vec![0x06, 0x51, 0x00, 0x00, 0x00, 0x08, 0x00],
                    ),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                // POSITIVE CONTROL -- correct today, must stay correct after the fix.
                let alert = bq.mac_safety_alert().dispatch_async().await.unwrap();
                assert!(
                    alert.ptos(),
                    "CONTROL: SLUUCN4B 16.1.37 DOES define PTOS at SafetyAlert bit 19. \
                     If this assertion ever fails, a SafetyStatus fix was over-applied \
                     to SafetyAlert."
                );

                let safety = bq.mac_safety_status().dispatch_async().await.unwrap();
                assert!(
                    !safety.ptos(),
                    "TRM-correct behaviour: SafetyStatus bit 19 is \"RSVD (Bit 19): \
                     Reserved. Do not use.\" on R3/R4/R5 -- SLUUCN4B 16.1.38 -- so \
                     there must be no ptos() on SafetyStatus. Note this is asymmetric \
                     with SafetyAlert, which keeps PTOS at bit 19."
                );

                bq.interface.i2c.done();
            }

            // ---------------- C2..C6: transport ----------------

            /// Finding C2 (critical): the `smbus_pec::Pec` accumulator in
            /// `read_with_retries` is constructed OUTSIDE the retry loop.
            /// `core::hash::Hasher::finish` does not reset it, so the second
            /// attempt's CRC continues from the first attempt's residue instead of
            /// restarting from the address/command preamble. A frame carrying the
            /// WRONG PEC is therefore accepted on retry, which defeats the entire
            /// point of PEC.
            ///
            /// SMBus PEC is CRC-8 (poly 0x07, init 0x00) over the whole
            /// transaction including the address bytes. SLUSBS8B and all four TRMs
            /// delegate the definition to the SMBus specification at smbus.org and
            /// never restate it, so there is no TRM sentence to quote; the
            /// invariant under test is simply that the CRC is per-transaction.
            ///
            /// For a 2-byte read of register 0x16 returning `40 00`, the only
            /// correct PEC is CRC-8 over [0x16, 0x16, 0x17, 0x40, 0x00] = 0x85.
            /// The frames below all carry 0xAC, which is wrong on every attempt.
            /// A conforming driver rejects all four and returns BQ40Z50Error::Pec.
            ///
            /// Offending code: src/interface.rs:149 (accumulator hoisted above the
            /// `loop` at :166), and the embassy-timeout copy at :524 / :541.
            ///
            /// NOTE: already addressed by open PR #61. This test should go green
            /// the moment #61 merges; it is kept so the fix has a regression guard.
            ///
            /// PATTERN A -- no signature or accessor changes; the test needs no
            /// edit when fixed.
            #[tokio::test]
            async fn finding_c2_pec_must_be_recomputed_on_every_retry() {
                let attempt = || Transaction::write_read(BQ_ADDR, vec![0x16], vec![0x40, 0x00, 0xAC]);
                // 1 initial attempt + 3 retries.
                let expectations = vec![attempt(), attempt(), attempt(), attempt()];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new_with_config(
                    i2c,
                    NoopDelay::new(),
                    Config {
                        pec_read: true,
                        ..Default::default()
                    },
                );

                let res = bq.device.battery_status().read_async().await;

                assert!(
                    res.is_err(),
                    "TRM/SMBus-correct behaviour: the only valid PEC for this frame is \
                     0x85 on EVERY attempt, so a frame carrying 0xAC must be rejected \
                     every time. The driver accepts it on attempt 2 because the CRC \
                     accumulator is never reset between retries."
                );

                bq.device.interface.i2c.done();
            }

            /// Finding C3 (critical): when a data-flash block read fails its PEC
            /// check, the retry path `continue`s the INNER chunk loop without
            /// re-sending the starting address. The gauge auto-increments its DF
            /// read pointer, so the retry reads the NEXT 32-byte block and the
            /// driver hands that block back to the caller as if it were the block
            /// that was requested. A CRC failure is converted into silent data
            /// corruption -- the exact scenario PEC exists to prevent.
            ///
            /// SLUUCN4B (BQ40Z50-R5 TRM) §16.1.101 "ManufacturerAccess()
            /// 0x4000-0x5FFF DataFlashAccess": "The gauge supports an auto-increment
            /// on the address during a DF read. [...] If another SMBus read block is
            /// sent with command 0x44, the gauge returns another 32 bytes of DF
            /// data, starting with address 0x4020."
            ///
            /// Because the read pointer has already advanced, a retry MUST re-send
            /// the block write that sets the starting address. The mock below
            /// encodes exactly that conforming sequence: set address, bad block,
            /// SET ADDRESS AGAIN, good block.
            ///
            /// Offending code: src/interface.rs:441-447, and the embassy-timeout
            /// copy at :859-866.
            ///
            /// PATTERN A -- no signature changes. FAILS TODAY by mock panic: the
            /// driver skips the third transaction (the address re-send) and issues
            /// a bare read instead, so embedded-hal-mock reports an unexpected
            /// write_read where a write was expected. That panic IS the finding.
            #[tokio::test]
            async fn finding_c3_df_pec_retry_must_resend_the_starting_address() {
                let mut bad = vec![0x22u8, 0x00, 0x40];
                bad.extend_from_slice(&[0xAAu8; 32]);
                bad.push(0xEB); // corrupt; the correct PEC for this frame is 0x14
                let mut good = vec![0x22u8, 0x00, 0x40];
                good.extend_from_slice(&[0xAAu8; 32]);
                good.push(0x14); // valid

                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x00, 0x40, 0xAB]),
                    Transaction::write_read(BQ_ADDR, vec![0x44], bad),
                    // REQUIRED: rewind the gauge's auto-incrementing DF read pointer
                    // before retrying. The driver does not do this today.
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x00, 0x40, 0xAB]),
                    Transaction::write_read(BQ_ADDR, vec![0x44], good),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new_with_config(
                    i2c,
                    NoopDelay::new(),
                    Config {
                        pec_read: true,
                        ..Default::default()
                    },
                );

                let mut out = [0u8; 32];
                bq.read_dataflash(0x4000, &mut out).await.unwrap();

                assert_eq!(
                    out, [0xAAu8; 32],
                    "TRM-correct behaviour: the caller asked for the block at 0x4000, so \
                     that is the block that must come back. SLUUCN4B 16.1.101."
                );
                bq.device.interface.i2c.done();
            }

            /// Finding C4 (critical): `MAC_STOP_OUTPUT_CCADC_CAL` is declared at
            /// address 0x4481F0, i.e. MAC subcommand 0xF081 -- the command that
            /// STARTS raw ADC output. `MAC_OUTPUT_CCADC_CAL` has the same address.
            /// `allow_address_overlap: true` suppressed the duplicate-address check
            /// that would otherwise have caught this. A caller trying to leave
            /// calibration mode re-enters it.
            ///
            /// SLUUCN4B (BQ40Z50-R5 TRM) §14.2 "Calibration", ManufacturerAccess()
            /// table: "0xF080  Disables raw ADC data output on ManufacturerData()"
            /// and "0xF081  Outputs raw ADC data of voltage, current, and
            /// temperature on ManufacturerData()". The same table appears in
            /// SLUUA43A §12.1.61 "ManufacturerAccess() 0xF080 Exit Calibration
            /// Output Mode" / §12.1.62, SLUUBU5A §15.1.84, and SLUUCH2 §16.1.99
            /// "ManufacturerAccess() 0xF080 and 0xF081 Output CCADCCal Control".
            ///
            /// The identical defect applies to `MAC_STOP_OUTPUT_SHORTED_CCADC_CAL`
            /// at 0x4482F0 (= 0xF082, the shorted-input ENABLE).
            ///
            /// Offending manifest entries: device_r5.yaml:2376-2384 and :2440-2448;
            /// device_r1.yaml:1582, device_r3.yaml:1880, device_r4.yaml:2290.
            ///
            /// PATTERN B. The preferred fix is DELETION, not re-addressing: 0xF080
            /// is already reachable as `MAC_EXIT_CALIBRATION_OUTPUT_MODE` at
            /// 0x4480F0, so the two MAC_STOP_* entries are redundant as well as
            /// wrong. WHEN FIXED: if the entries are deleted, delete this test --
            /// `mac_exit_calibration_output_mode()` is already covered by the second
            /// half of it. If they are instead re-addressed to 0x4480F0, this test
            /// goes green unchanged.
            #[tokio::test]
            async fn finding_c4_stop_ccadc_cal_must_send_subcommand_f080() {
                let expectations = vec![
                    // TRM-correct: the DISABLE subcommand 0xF080, little endian.
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x80, 0xF0]),
                    // Control: the genuine disable, reachable under this other name.
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x80, 0xF0]),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                // FAILS TODAY: emits 0x81 0xF0 (the ENABLE subcommand) instead.
                bq.mac_stop_output_ccadc_cal().dispatch_async().await.unwrap();
                bq.mac_exit_calibration_output_mode()
                    .dispatch_async()
                    .await
                    .unwrap();

                bq.interface.i2c.done();
            }

            /// Finding C5 (critical): `AsyncBufferInterface::write` copies the
            /// caller's buffer into a 34-byte stack array and then passes `&data`
            /// -- the WHOLE array -- to `write_with_retries`, instead of
            /// `&data[..=buf.len()]`. A 3-byte buffer write puts 30 bytes of zero
            /// padding on the wire after the payload. The return value still
            /// reports the caller's length, so the padding is invisible from the
            /// API.
            ///
            /// The SMBus block protocol is length-prefixed and no SLUUCN4B register
            /// accepts trailing padding, so there is no TRM sentence to quote --
            /// no TRM contemplates this. The defect is self-evident against the
            /// sibling `write_register` at src/interface.rs:902, which correctly
            /// slices `&buf[..=data.len()]`.
            ///
            /// Offending code: src/interface.rs:977.
            ///
            /// PATTERN A -- no signature changes. FAILS TODAY by mock panic: the
            /// driver puts 34 bytes on the bus where 4 were expected.
            #[tokio::test]
            async fn finding_c5_buffer_write_must_emit_only_the_payload() {
                use device_driver::AsyncBufferInterface;
                // TRM-correct: command byte plus exactly the 3 payload bytes.
                let expectations = vec![Transaction::write(BQ_ADDR, vec![0x2F, 0x01, 0x02, 0x03])];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                let n = bq
                    .device
                    .interface
                    .write(0x2F, &[0x01, 0x02, 0x03])
                    .await
                    .unwrap();

                assert_eq!(
                    n, 3,
                    "correct behaviour: three bytes accepted and three bytes transmitted, \
                     with no stack padding trailing the payload"
                );
                bq.device.interface.i2c.done();
            }

            /// Finding C6 (critical): MAC block-read responses are never validated
            /// against the request. `mac_read_with_retries` skips exactly three
            /// bytes -- the length byte and the two-byte command echo -- and copies
            /// the remainder out. It never compares the echoed command to the
            /// command it sent, nor the length byte to the expected payload size.
            /// A stale or misaddressed response is indistinguishable from a
            /// correct one.
            ///
            /// SLUUCN4B (BQ40Z50-R5 TRM) §16.1 "0x00 ManufacturerAccess() and 0x44
            /// ManufacturerBlockAccess()": "SMBus block read. Command = 0x44. Data
            /// read = 06 00 00 01 [...] The first 2 bytes, '06 00', is the MAC
            /// command." The echo exists precisely so the host can confirm which
            /// command it is looking at.
            ///
            /// Below, DeviceType (0x0001) is requested and the gauge answers with
            /// an echo of 0x9999. That is not a valid response to this request and
            /// must not reach the caller.
            ///
            /// Offending code: src/interface.rs:284-287, and the embassy-timeout
            /// copy at :676-679.
            ///
            /// PATTERN A for the call site. Note that `BQ40Z50Error` has no variant
            /// for a command mismatch, so the fix needs a new variant too; the
            /// assertion is deliberately only `is_err()` so that it stays valid
            /// whatever the variant ends up being called. Tighten it once the
            /// variant exists.
            #[tokio::test]
            async fn finding_c6_mismatched_mac_command_echo_must_be_an_error() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x01, 0x00]),
                    Transaction::write_read(BQ_ADDR, vec![0x44], vec![0x04, 0x99, 0x99, 0x34, 0x12]),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                let res = bq.mac_device_type().dispatch_async().await;

                assert!(
                    res.is_err(),
                    "TRM-correct behaviour: DeviceType (0x0001) was requested, so a \
                     response echoing MAC command 0x9999 must be rejected, not reported \
                     as success. SLUUCN4B 16.1."
                );

                bq.interface.i2c.done();
            }

            // ---------------- M1..M9: API and manifest ----------------

            /// Finding M1 (major): `write_authentication_key` takes
            /// `&[u8; AUTH_KEY_LEN_BYTES]` = `&[u8; 18]` and writes a length byte of
            /// 18, but then emits two command bytes PLUS eighteen key bytes -- twenty
            /// bytes after the length byte. The frame contradicts itself and the key
            /// is the wrong size. The sibling `read_authentication_key` gets it
            /// right, using `AUTH_KEY_DATA_LEN_BYTES` (16).
            ///
            /// SLUUCN4B (BQ40Z50-R5 TRM) §16.1.35 "ManufacturerAccess() 0x0037
            /// Authentication Key": "Send the AuthenticationKey() + the new 128-bit
            /// authentication key to ManufacturerBlockAccess()". 128 bits is 16
            /// bytes. SLUUA43A §12.1.34, SLUUBU5A §15.1.34 and SLUUCH2 §16.1.34
            /// carry the same wording.
            ///
            /// The TRM-correct frame is therefore `44 12 37 00` followed by exactly
            /// 16 key bytes -- 20 bytes total, with the length byte 0x12 = 18 =
            /// 2 command bytes + 16 key bytes. That is what the mock expects.
            ///
            /// Offending code: src/versions/r5.rs:137-146 and the identical bodies
            /// in r1.rs / r3.rs / r4.rs; src/consts.rs:16-18.
            ///
            /// PATTERN D -- the call site below uses today's `&[u8; 18]` signature
            /// so that this file compiles; the assertion (the expected wire frame)
            /// is already the TRM-correct one. FAILS TODAY by mock panic: 22 bytes
            /// are transmitted where 20 were expected.
            /// WHEN FIXED: the signature becomes `&[u8; 16]`; change the argument
            /// below to `&[0xAA; 16]`. The expectation needs no change.
            #[tokio::test]
            async fn finding_m1_auth_key_frame_must_be_16_key_bytes() {
                let mut frame = vec![0x44u8, 0x12, 0x37, 0x00];
                frame.extend_from_slice(&[0xAAu8; 16]);
                assert_eq!(
                    frame.len(),
                    20,
                    "SLUUCN4B 16.1.35: 128-bit key, 20 bytes on the wire"
                );

                let expectations = vec![Transaction::write(BQ_ADDR, frame)];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                bq.write_authentication_key(&[0xAA; 18]).await.unwrap();
                bq.device.interface.i2c.done();
            }

            /// Finding M2 (major): `SECURITY_KEYS_DATA_LEN_BYTES` is a single
            /// constant hard-coded to 8 and shared by four revisions that disagree
            /// about the size of the Security Keys block.
            ///
            ///   SLUUA43A §12.1.33 "ManufacturerAccess() 0x0035 Security Keys"
            ///     (R1, UNSEAL + FULL ACCESS): "Data = MAC command + New UNSEAL key
            ///     + New FULL ACCESS KEY = 35 00 34 12 78 56 FF FF FF FF"
            ///                                                    ->  8 key bytes
            ///   SLUUBU5A §15.1.33 (R3, + Manual PF + Lifetimes Reset):
            ///     "= 35 00 34 12 78 56 FF FF FF FF 57 28 98 2A 14 2B 8A 2C"
            ///                                                    -> 16 key bytes
            ///   SLUUCH2  §16.1.33 (R4, + DF Read Only + Override)
            ///                                                    -> 24 key bytes
            ///   SLUUCN4B §16.1.34 (R5, + MfgInfoC Write):
            ///     "= 35 00 34 12 78 56 FF FF FF FF 32 76 12 17 57 28 98 2A 14 2B
            ///        8A 2C 18 2D 9B 2E 45 3C 89 5D"               -> 28 key bytes
            ///
            /// So the driver's 8 is CORRECT FOR R1 and this test is cfg'd off
            /// there. On R3 half the keys are unreachable, on R4 two thirds, on R5
            /// just over two thirds. `read_security_keys` has the identical defect
            /// (it reads 8 bytes into a `[u8; 8]`); it is not separately tested
            /// because the same constant fix covers both.
            ///
            /// Offending code: src/consts.rs:13, consumed by src/versions/r5.rs:88
            /// (write) and :64 (read), and the r3/r4 equivalents.
            ///
            /// PATTERN D -- the call site passes today's `&[u8; 8]`; the expected
            /// frame is the TRM-correct, revision-specific one. FAILS TODAY by mock
            /// panic on both the length byte and the frame length.
            /// WHEN FIXED: `SECURITY_KEYS_DATA_LEN_BYTES` becomes revision-specific
            /// (16/24/28); change the argument below to `&[0xAA; KEY_BYTES]`. The
            /// expectation needs no change.
            #[cfg(not(feature = "r1"))]
            #[tokio::test]
            async fn finding_m2_security_keys_block_must_match_the_revision() {
                #[cfg(feature = "r3")]
                const KEY_BYTES: usize = 16; // SLUUBU5A 15.1.33
                #[cfg(feature = "r4")]
                const KEY_BYTES: usize = 24; // SLUUCH2 16.1.33
                #[cfg(feature = "r5")]
                const KEY_BYTES: usize = 28; // SLUUCN4B 16.1.34

                // MAC block write: 0x44, length byte, then 0x35 0x00 + the key bytes.
                let len_byte = u8::try_from(KEY_BYTES + 2).unwrap();
                let mut frame = vec![0x44u8, len_byte, 0x35, 0x00];
                frame.extend_from_slice(&[0xAAu8; KEY_BYTES]);

                let expectations = vec![Transaction::write(BQ_ADDR, frame)];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                bq.write_security_keys(&[0xAA; 8]).await.unwrap();

                bq.device.interface.i2c.done();
            }

            /// Finding M4 (major): `set_battery_mode` mutates the cached
            /// capacity-mode state BEFORE issuing the write and never rolls it back
            /// on failure. After a write that was NAK'd on all four attempts the
            /// cache says centiwatts while the part is still in milliamps, and
            /// every subsequent `remaining_capacity()`, `full_charge_capacity()`,
            /// `design_capacity()` and `at_rate()` is silently tagged with the
            /// wrong unit.
            ///
            /// SLUUCN4B (BQ40Z50-R5 TRM) §16.4 "0x03 BatteryMode()": the gauge's
            /// reporting unit is whatever is latched in the CAPACITY_MODE bit of
            /// that register. A write that was never acknowledged latched nothing,
            /// so the cache must still reflect milliamps.
            ///
            /// Offending code: src/common.rs:156-162 --
            /// `self.set_capacity_mode_state(flags)` precedes
            /// `self.device.battery_mode().write_async(...)` with no rollback.
            /// Related: both constructors assume `CapacityModeState::Milliamps`
            /// without ever reading BatteryMode() from the part.
            ///
            /// PATTERN A -- no signature or accessor changes; needs no edit when
            /// fixed.
            #[tokio::test]
            async fn finding_m4_failed_battery_mode_write_must_not_change_the_cached_unit() {
                let nak = || {
                    Transaction::write(BQ_ADDR, vec![0x03, 0x00, 0x80]).with_error(
                        embedded_hal::i2c::ErrorKind::NoAcknowledge(embedded_hal::i2c::NoAcknowledgeSource::Address),
                    )
                };
                // 1 initial attempt + 3 retries.
                let expectations = vec![nak(), nak(), nak(), nak()];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                assert_eq!(bq.capacity_mode_state.get(), CapacityModeState::Milliamps);

                let res = bq
                    .set_battery_mode(BatteryModeFields::new().with_capacity_mode(true))
                    .await;
                assert!(res.is_err(), "the write must fail for this test to mean anything");

                assert_eq!(
                    bq.capacity_mode_state.get(),
                    CapacityModeState::Milliamps,
                    "correct behaviour: the write was NAK'd four times, so BatteryMode() \
                     [CAPACITY_MODE] was never latched (SLUUCN4B 16.4) and the cached unit \
                     must still be milliamps. The driver caches the new unit before the \
                     write and never rolls it back, so it now mislabels every capacity \
                     reading as centiwatts."
                );

                bq.device.interface.i2c.done();
            }

            /// Finding M5 (major): `set_remaining_capacity_alarm` and `set_at_rate`
            /// collapse the two `CapacityModeValue` variants into a single match arm
            /// and write the raw number:
            ///
            ///     MilliAmpUnsigned(value) | CentiWattUnsigned(value) => value
            ///
            /// A caller passing centiwatt-hours while the gauge is in milliamp-hour
            /// mode programs an mAh alarm with a cWh magnitude and gets no error.
            /// The enum exists precisely to carry that distinction.
            ///
            /// SLUUCN4B (BQ40Z50-R5 TRM) §16.2 "0x01 RemainingCapacityAlarm()" and
            /// §16.5 "0x04 AtRate()": the register's unit is selected by
            /// BatteryMode()[CAPACITY_MODE] (§16.4), not by the caller.
            ///
            /// The driver cannot convert cWh to mAh (it would need the pack
            /// voltage), so the only sound behaviour is to reject the mismatch.
            /// That is what this test asserts.
            ///
            /// Offending code: src/common.rs:128-131 and :179-182.
            ///
            /// PATTERN A -- no signature changes. The mock is pre-loaded with the
            /// frame the driver wrongly emits so that the failure surfaces as this
            /// test's own assertion rather than as a mock panic; `done()` is
            /// deliberately not called, because a conforming driver leaves that
            /// expectation unconsumed.
            #[tokio::test]
            async fn finding_m5_capacity_alarm_must_reject_a_mismatched_unit() {
                // The frame the driver wrongly emits: 500 = 0x01F4 LE to register 0x01.
                let expectations = vec![Transaction::write(BQ_ADDR, vec![0x01, 0xF4, 0x01])];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                assert_eq!(
                    bq.capacity_mode_state.get(),
                    CapacityModeState::Milliamps,
                    "precondition: the driver believes the gauge is reporting mAh"
                );

                let res = bq
                    .set_remaining_capacity_alarm(CapacityModeValue::CentiWattUnsigned(500))
                    .await;

                assert!(
                    res.is_err(),
                    "correct behaviour: the gauge is in mAh mode (BatteryMode()\
                     [CAPACITY_MODE] = 0, SLUUCN4B 16.4) and the caller supplied \
                     centiwatt-hours. The driver cannot convert without the pack voltage, \
                     so it must reject the call. Today it strips the unit tag and writes \
                     the raw 500 as an mAh alarm."
                );
            }

            /// Finding M7 (major): public trait entry points guard length with
            /// `debug_assert!`, which is compiled out in release, and then index
            /// past a fixed backing array. On a no_std target a caller-supplied
            /// length becomes a panic (debug) or an out-of-bounds slice (release)
            /// rather than an error. The `BQ40Z50Error::DataTooLarge` variant
            /// already exists and is used correctly by `write_mfg_info_c`
            /// (src/versions/r5.rs:288).
            ///
            /// No TRM sentence applies; this is an API-robustness defect judged
            /// against the crate's own existing error variant.
            ///
            /// Offending code: src/interface.rs:892 (`write_register`), :912
            /// (`read_register`), :948 (`dispatch_command`), :970
            /// (`AsyncBufferInterface::write`).
            ///
            /// PATTERN E -- asserts the Err that should be returned. FAILS TODAY by
            /// panicking at the `debug_assert!("Buffer size too big")` inside the
            /// driver, before this test's own assertion is reached. That panic IS
            /// the finding: a caller length must not be able to abort the firmware.
            #[tokio::test]
            async fn finding_m7_oversized_buffer_write_must_return_data_too_large() {
                use device_driver::AsyncBufferInterface;
                let i2c = Mock::new(&[]);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                // LARGEST_BUF_SIZE_BYTES is 33. 40 bytes is a plain caller error.
                let res = bq.device.interface.write(0x2F, &[0xAAu8; 40]).await;

                assert!(
                    matches!(res, Err(BQ40Z50Error::DataTooLarge)),
                    "correct behaviour: an oversized caller buffer must come back as \
                     BQ40Z50Error::DataTooLarge, the variant this crate already defines \
                     and already uses in write_mfg_info_c. It must never panic or index \
                     out of bounds on a no_std target."
                );
            }

            /// Finding M8 (major): the data-flash starting address is never
            /// validated against the documented 0x4000-0x5FFF window. The driver's
            /// own doc comment states the range ("Starting address should be
            /// between 0x4000 and 0x5FFF", src/versions/r5.rs:462) but nothing
            /// enforces it, so a caller can write over whatever lives outside it.
            ///
            /// SLUUCN4B (BQ40Z50-R5 TRM) §16.1.101 "ManufacturerAccess()
            /// 0x4000-0x5FFF DataFlashAccess" -- the section title states the valid
            /// address range.
            ///
            /// Offending code: src/interface.rs:63-66.
            ///
            /// PATTERN A -- no signature changes. The mock is pre-loaded with the
            /// frame the driver wrongly emits so that the failure surfaces as this
            /// test's own assertion rather than as a mock panic; `done()` is
            /// deliberately not called, because a conforming driver rejects the
            /// call before touching the bus.
            #[tokio::test]
            async fn finding_m8_df_address_outside_the_window_must_be_rejected() {
                // The frame the driver wrongly emits for an out-of-window address.
                let mut chunk = vec![0x44u8, 34, 0x00, 0x60];
                chunk.extend_from_slice(&[0xAAu8; 32]);
                let expectations = vec![Transaction::write(BQ_ADDR, chunk)];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                let res = bq.write_dataflash(0x6000, &[0xAAu8; 32]).await;

                assert!(
                    res.is_err(),
                    "correct behaviour: 0x6000 is outside the documented data-flash \
                     window 0x4000-0x5FFF (SLUUCN4B 16.1.101), so the write must be \
                     rejected before anything reaches the bus. Today it is transmitted."
                );
            }

            /// Finding M8 (major), continued: because the starting address is
            /// unchecked, the per-chunk address `starting_address + start_idx as u16`
            /// can overflow. An address near the top of the u16 space with a
            /// multi-chunk payload panics in debug and silently wraps to a LOW
            /// address in release -- which on a fuel gauge means writing over
            /// whatever data-flash item lives there.
            ///
            /// Note also that the first 32-byte chunk is committed to the bus
            /// BEFORE the overflow is hit on the second, which demonstrates finding
            /// Md2 ("DF writes commit chunks with no rollback") in the same shot.
            ///
            /// SLUUCN4B §16.1.101 "ManufacturerAccess() 0x4000-0x5FFF
            /// DataFlashAccess".
            ///
            /// Offending code: src/interface.rs:63-66.
            ///
            /// PATTERN E -- asserts the Err that should be returned. FAILS TODAY by
            /// panicking with "attempt to add with overflow" inside the driver,
            /// before this test's own assertion is reached. That panic IS the
            /// finding.
            #[tokio::test]
            async fn finding_m8_df_address_overflow_must_be_rejected_not_panic() {
                // Chunk 1 at 0xFFF0 is transmitted; chunk 2 computes 0xFFF0 + 32.
                let mut chunk1 = vec![0x44u8, 34, 0xF0, 0xFF];
                chunk1.extend_from_slice(&[0xAAu8; 32]);
                let expectations = vec![Transaction::write(BQ_ADDR, chunk1)];
                let i2c = Mock::new(&expectations);
                let mut bq = Bq40z50::new(i2c, NoopDelay::new());

                let res = bq.write_dataflash(0xFFF0, &[0xAAu8; 64]).await;

                assert!(
                    res.is_err(),
                    "correct behaviour: 0xFFF0 is outside the 0x4000-0x5FFF data-flash \
                     window (SLUUCN4B 16.1.101) and its second chunk address overflows \
                     u16. This must be an Err, never a panic and never a silent wrap to \
                     a low data-flash address."
                );
            }

            /// Finding M9 (major): `MAC_CHEM_ID` is declared `size_bits_out: 8`, so
            /// the driver sizes its read at 1 length byte + 2 command echo + 1 data
            /// byte and returns only the low byte of a 16-bit chemistry ID.
            ///
            /// SLUUCN4B (BQ40Z50-R5 TRM) §16.1 "0x00 ManufacturerAccess() and 0x44
            /// ManufacturerBlockAccess()", worked example: "SMBus block read.
            /// Command = 0x44. Data read = 06 00 00 01 [...] The second 2 bytes,
            /// '00 01', is the chem ID returning in little endian. That is 0x0100,
            /// chem ID 100." SLUUA43A §12.1 carries the identical example ("That is
            /// 0x0100, chem ID 100"). The field is 16 bits.
            ///
            /// The mock below is the TRM's own example response, length-prefixed:
            /// `04` (2 echo + 2 data), `06 00` (the echo), `00 01` (the chem ID).
            ///
            /// Offending manifest entry: device_r5.yaml:85; same line in
            /// device_r1.yaml, device_r3.yaml, device_r4.yaml.
            ///
            /// PATTERN A -- the `u16::from(...)` wrapper on the assertion compiles
            /// both against today's `u8` accessor and against the `u16` one the fix
            /// produces, so this test needs NO edit when fixed. FAILS TODAY by mock
            /// panic: the driver clocks 4 bytes where the TRM's example is 5.
            #[tokio::test]
            async fn finding_m9_chem_id_must_be_16_bits() {
                let expectations = vec![
                    Transaction::write(BQ_ADDR, vec![0x44, 0x02, 0x06, 0x00]),
                    // The TRM's own worked example: 06 00 00 01, length-prefixed.
                    Transaction::write_read(BQ_ADDR, vec![0x44], vec![0x04, 0x06, 0x00, 0x00, 0x01]),
                ];
                let i2c = Mock::new(&expectations);
                let mut bq = Device::new(DeviceInterface::new(i2c, NoopDelay::new()));

                let out = bq.mac_chem_id().dispatch_async().await.unwrap();

                assert_eq!(
                    u16::from(out.chem_id()),
                    0x0100,
                    "TRM-correct behaviour: SLUUCN4B 16.1's worked example reads back \
                     chem ID 0x0100. The manifest declares size_bits_out: 8, so the high \
                     byte is never clocked in and the ID is truncated."
                );

                bq.interface.i2c.done();
            }
        }
    };
}

pub(crate) use bq40z50_tests;
