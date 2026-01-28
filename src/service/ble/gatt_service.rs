use embedded_storage_async::nor_flash::MultiwriteNorFlash;
use log::{error, info};
use sequential_storage::{cache::NoCache, map::MapStorage};
use trouble_host::{BondInformation, gatt::{GattConnection, GattConnectionEvent, GattEvent, ReadEvent, WriteEvent}, prelude::{DefaultPacketPool, SecurityLevel}};

use crate::service::ble::storage_service::{self, StorageAddr};


pub struct GattService {
    notification_counter: u32,
}

impl GattService {
    pub fn new() -> Self {
        GattService {
            notification_counter: 0,
        }
    }

    pub fn handle_disconect_event(&self) {
        // Handle disconnection event
    }

    pub async fn handle_paring_complete_event<S: MultiwriteNorFlash>(
        &self, 
        security_level: SecurityLevel, 
        bond: BondInformation, 
        storage: &mut MapStorage<StorageAddr, S, NoCache>,
    ) -> bool {
        info!("[gatt] pairing complete: {:?}", security_level);

        match storage_service::store_bonding_info(storage, &bond).await {
            Ok(_) => {
                info!("[gatt] Bonding information stored successfully");
                true
            },
            Err(e) => {
                error!("[gatt] Failed to store bonding information: {:?}", e);
                false
            }
        }
    }

    pub fn handle_paring_failed_event(&self, err: trouble_host::Error) {
        error!("[gatt] pairing failed {:?}", err);
    }

    pub fn handle_gatt_event<'stack, 'server>(
        &self, 
        gatt_event: GattEvent<'stack, 'server, DefaultPacketPool>, 
        conn: &GattConnection<'_, '_, DefaultPacketPool>
    ) {
        match gatt_event {
            GattEvent::Write(event) => {
                self.gatt_write_handler(event, conn);
            },
            GattEvent::Read(event) =>{
                self.gatt_read_handler(event);
            }
            _ => {}
        }
    }

    fn gatt_write_handler<'stack, 'server>(
        &self, 
        event: WriteEvent<'stack, 'server, DefaultPacketPool>,
            conn: &GattConnection<'_, '_, DefaultPacketPool>
    ) {
        match conn.raw().security_level() {
            core::prelude::v1::Ok(SecurityLevel::NoEncryption) => {
                error!("[gatt] Write operation rejected: Connection is not encrypted");
            },
            core::prelude::v1::Ok(SecurityLevel::Encrypted) | core::prelude::v1::Ok(SecurityLevel::EncryptedAuthenticated) => {
                info!("[gatt] Write operation accepted");
                let value = event.payload().handle();
                
                info!("[gatt] Written data: {:?}", value);

                match event.accept() {
                    core::prelude::v1::Ok(data) => {                        
                        let _ = data.try_send();
                    },
                    core::prelude::v1::Err(e) => {
                        error!("[gatt] Failed to accept write: {:?}", e);
                    }
                }

            },
            core::prelude::v1::Err(e) => {
                error!("[gatt] Write operation rejected: Failed to get security level: {:?}", e);
            }
        }      
    }


    fn gatt_read_handler<'stack, 'server>(&self, event: ReadEvent<'stack, 'server, DefaultPacketPool>) {
        info!("[gatt] Read request received on handle: {:?}", event.payload().handle());
        // Você pode inspecionar qual característica está sendo lida
        // e responder de acordo
    }

    /// Envia uma notificação de exemplo através do Battery Service
    /// Esta é uma função de demonstração para mostrar como enviar notificações
    // pub async fn send_example_notification(&mut self, _conn: &GattConnection<'_, '_, DefaultPacketPool>) {
    //     self.notification_counter += 1;
    //     info!("[gatt] Sending notification #{}", self.notification_counter);
    //     // Para enviar notificações, você precisará ter uma referência ao serviço específico
    //     // Por exemplo, battery_service.level_notify(conn).await
    // }

    pub async fn handle_gatt_events<S: MultiwriteNorFlash>(
        &self,
        storage: &mut MapStorage<StorageAddr, S, NoCache>,
        //server: &Connection<'_, DefaultPacketPool>,
        conn: &GattConnection<'_, '_, DefaultPacketPool>,
        bond_stored: &mut bool,
    ) {
        let reason = loop {
            match conn.next().await{
                GattConnectionEvent::Disconnected {reason} => { break reason }
                GattConnectionEvent::PairingComplete {
                    security_level, 
                    bond
                } => {
                    info!("[gatt] pairing complete: {:?}", security_level);

                    if let Some(bond) = bond {
                        
                        *bond_stored = self.handle_paring_complete_event(
                            security_level, 
                            bond,
                            storage
                        ).await;
                    }
                },
                GattConnectionEvent::PairingFailed(err) => {
                    self.handle_paring_failed_event(err);
                },
                GattConnectionEvent::Gatt { event } => {
                    self.handle_gatt_event(event, conn);
                },
                _ => {  
                    // Handle other events if necessary
                }
            }
        };
        info!("[gatt] disconnected: {:?}", reason);
    }
}