use embedded_storage_async::nor_flash::{MultiwriteNorFlash, NorFlash};
use trouble_host::prelude::{BdAddr, SecurityLevel};
use trouble_host::{BondInformation, Identity, LongTermKey};
use sequential_storage::cache::NoCache;
use sequential_storage::map::{Key, MapConfig, MapStorage, SerializationError, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageAddr(BdAddr);

impl Key for StorageAddr {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        if buffer.len() < 6 {
            return Err(SerializationError::BufferTooSmall);
        }
        buffer[0..6].copy_from_slice(self.0.raw());
        Ok(6)
    }

    fn deserialize_from(buffer: &[u8]) -> Result<(Self, usize), SerializationError> {
        if buffer.len() < 6 {
            Err(SerializationError::BufferTooSmall)
        } else {
            Ok((StorageAddr(BdAddr::new(buffer[0..6].try_into().unwrap())), 6))
        }
    }
}

pub struct StoredBondInformation {
    ltk: LongTermKey,
    security_level: SecurityLevel,
}

impl<'a> Value<'a> for StoredBondInformation {
    fn serialize_into(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
        if buffer.len() < 17 {
            return Err(SerializationError::BufferTooSmall);
        }
        buffer[0..16].copy_from_slice(self.ltk.to_le_bytes().as_slice());
        buffer[16] = match self.security_level {
            SecurityLevel::NoEncryption => 0,
            SecurityLevel::Encrypted => 1,
            SecurityLevel::EncryptedAuthenticated => 2,
        };
        Ok(17)
    }

    fn deserialize_from(buffer: &'a [u8]) -> Result<(Self, usize), SerializationError>
    where
        Self: Sized,
    {
        if buffer.len() < 17 {
            Err(SerializationError::BufferTooSmall)
        } else {
            let ltk = LongTermKey::from_le_bytes(buffer[0..16].try_into().unwrap());
            let security_level = match buffer[16] {
                0 => SecurityLevel::NoEncryption,
                1 => SecurityLevel::Encrypted,
                2 => SecurityLevel::EncryptedAuthenticated,
                _ => return Err(SerializationError::InvalidData),
            };
            Ok((StoredBondInformation { ltk, security_level }, 17))
        }
    }
}

async fn store_bonding_info<S: MultiwriteNorFlash>(
    storage: &mut MapStorage<StorageAddr, S, NoCache>,
    info: &BondInformation,
) -> Result<(), sequential_storage::Error<S::Error>> {
    let mut buffer = [0; 32];
    let key = StorageAddr(info.identity.bd_addr);
    
    // "Manually cloning" to avoid lifetime issues
    let value = StoredBondInformation {
        ltk: info.ltk,
        security_level: info.security_level,
    };

    match storage.remove_item(&mut buffer, &key).await {
        Ok(_) => {}
        Err(e) => return Err(e),
    }

    storage.store_item(&mut buffer, &key, &value).await
}

async fn load_bonding_info<S: MultiwriteNorFlash>(
    storage: &mut MapStorage<StorageAddr, S, NoCache>,
    addr: &BdAddr,
) -> Result<Option<BondInformation>, sequential_storage::Error<S::Error>> {
    let mut buffer = [0; 32];
    let key = StorageAddr(*addr);

    match storage.fetch_item::<StoredBondInformation>(&mut buffer, &key).await {
        Ok(Some(stored)) => {
            // Convert StoredBondInformation back to BondInformation
            let bond_info = BondInformation {
                identity: Identity {
                    bd_addr: *addr,
                    irk: None,
                },
                ltk: stored.ltk,
                security_level: stored.security_level,
                is_bonded: true,
            };
            Ok(Some(bond_info))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn init_storage<S: MultiwriteNorFlash>(flash: S) -> MapStorage<StorageAddr, S, NoCache> {
    let map_config = MapConfig::new(0x1000..0x3000);

    MapStorage::new(flash, map_config, NoCache {})
}