use log::{info, error};
use trouble_host::prelude::{GattConnection, DefaultPacketPool};
use crate::service::ble::notification_characteristics::{NotificationCharacteristics, TamagotchiStatus};

/// Helper para enviar notificações facilmente através do NotificationService
pub struct NotificationService;

impl NotificationService {
    /// Envia uma mensagem de texto para o telefone
    /// Retorna Ok(()) se enviado com sucesso
    pub async fn send_message(
        service: &NotificationCharacteristics,
        conn: &GattConnection<'_, '_, DefaultPacketPool>,
        message: &[u8],
    ) -> Result<(), trouble_host::Error> {
        if message.len() > 128 {
            error!("[notify] Message too long: {} bytes (max 128)", message.len());
            return Err(trouble_host::Error::InvalidValue);
        }

        let mut buffer = [0u8; 128];
        buffer[..message.len()].copy_from_slice(message);

        match service.message.notify(conn, &buffer).await {
            Ok(_) => {
                if let Ok(msg_str) = core::str::from_utf8(message) {
                    info!("[notify] Message sent: {}", msg_str);
                } else {
                    info!("[notify] Message sent: {:?}", message);
                }
                Ok(())
            }
            Err(e) => {
                error!("[notify] Failed to send message: {:?}", e);
                Err(e)
            }
        }
    }

    /// Envia atualização do contador
    pub async fn send_counter(
        service: &NotificationCharacteristics,
        conn: &GattConnection<'_, '_, DefaultPacketPool>,
        value: u32,
    ) -> Result<(), trouble_host::Error> {
        match service.counter.notify(conn, &value).await {
            Ok(_) => {
                info!("[notify] Counter sent: {}", value);
                Ok(())
            }
            Err(e) => {
                error!("[notify] Failed to send counter: {:?}", e);
                Err(e)
            }
        }
    }

    /// Envia atualização de status
    pub async fn send_status(
        service: &NotificationCharacteristics,
        conn: &GattConnection<'_, '_, DefaultPacketPool>,
        status: u8,
    ) -> Result<(), trouble_host::Error> {
        match service.tamagotchi_status.notify(conn, &status).await {
            Ok(_) => {
                info!("[notify] Status sent: {}", status);
                Ok(())
            }
            Err(e) => {
                error!("[notify] Failed to send status: {:?}", e);
                Err(e)
            }
        }
    }

    /// Envia status do Tamagotchi com mensagem automática
    pub async fn send_tamagotchi_status(
        service: &NotificationCharacteristics,
        conn: &GattConnection<'_, '_, DefaultPacketPool>,
        status: TamagotchiStatus,
    ) -> Result<(), trouble_host::Error> {
        // Envia o código de status
        Self::send_status(service, conn, status as u8).await?;
        
        // Envia a mensagem correspondente
        let message = status.as_message();
        Self::send_message(service, conn, message).await
    }
}