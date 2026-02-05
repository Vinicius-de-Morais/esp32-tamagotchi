use trouble_host::prelude::gatt_service;

/// Serviço customizado para enviar notificações/mensagens para o telefone
#[gatt_service(uuid = "12345678-1234-5678-1234-56789abcdef0")]
pub struct NotificationCharacteristics {
    /// Característica para enviar mensagens de texto
    #[characteristic(uuid = "12345678-1234-5678-1234-56789abcdef1", read, notify, value = [0u8; 128])]
    pub message: [u8; 128],
    
    /// Característica para contador de notificações
    #[characteristic(uuid = "12345678-1234-5678-1234-56789abcdef2", read, notify, value = 0)]
    pub counter: u32,
    
    /// Característica para status do Tamagotchi
    #[characteristic(uuid = "12345678-1234-5678-1234-56789abcdef3", read, notify, value = 0)]
    pub tamagotchi_status: u8,
}

/// Estados possíveis do Tamagotchi para notificações
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TamagotchiStatus {
    Happy = 0,
    Hungry = 1,
    Tired = 2,
    Sick = 3,
    Playing = 4,
    Sleeping = 5,
}

impl TamagotchiStatus {
    pub fn as_message(&self) -> &'static [u8] {
        match self {
            TamagotchiStatus::Happy => b"Estou feliz!",
            TamagotchiStatus::Hungry => b"Com fome...",
            TamagotchiStatus::Tired => b"Cansado...",
            TamagotchiStatus::Sick => b"Doente :(",
            TamagotchiStatus::Playing => b"Brincando!",
            TamagotchiStatus::Sleeping => b"Dormindo zzz",
        }
    }
}
