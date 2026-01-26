use network::client::IClientNetwork;
use network::messages::{ClientMessages, NetworkMessageType};
use network::NetworkClient;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct NetworkContainer {
    client_network: Arc<RwLock<NetworkClient>>,

    network_lock: Arc<AtomicBool>,
    timer: Arc<RwLock<Instant>>,

    runtime: Arc<std::sync::Mutex<Option<tokio::runtime::Runtime>>>,
}

/// Корректно завершает Tokio runtime при уничтожении контейнера.
///
/// Tokio runtime нельзя дропать из асинхронного контекста — это вызывает панику
/// "Cannot drop a runtime in a context where blocking is not allowed".
/// Godot может уничтожать объекты в async контексте, поэтому используется
/// `shutdown_background()` который не блокирует.
impl Drop for NetworkContainer {
    fn drop(&mut self) {
        if Arc::strong_count(&self.runtime) == 1 {
            if let Ok(mut guard) = self.runtime.lock() {
                if let Some(rt) = guard.take() {
                    rt.shutdown_background();
                }
            }
        }
    }
}

impl NetworkContainer {
    pub fn new(ip_port: String) -> Result<Self, String> {
        log::info!(target: "network", "Connecting to the server at &e{}", ip_port);

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(async { NetworkClient::new(ip_port).await });

        let network = match result {
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        Ok(Self {
            runtime: Arc::new(std::sync::Mutex::new(Some(runtime))),
            client_network: Arc::new(RwLock::new(network)),
            timer: Arc::new(RwLock::new(Instant::now())),
            network_lock: Arc::new(AtomicBool::new(false)),
        })
    }

    fn with_runtime<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&tokio::runtime::Runtime) -> R,
    {
        let guard = self.runtime.lock().unwrap();
        f(guard.as_ref().expect("Runtime already shutdown"))
    }

    pub(crate) fn get_client(&self) -> tokio::sync::RwLockReadGuard<'_, NetworkClient> {
        self.with_runtime(|rt| rt.block_on(async { self.client_network.read().await }))
    }

    pub fn send_message(&self, message_type: NetworkMessageType, message: &ClientMessages) {
        self.get_client().send_message(message_type, message);
    }

    pub fn is_network_locked(&self) -> bool {
        self.network_lock.load(Ordering::Relaxed)
    }

    pub fn set_network_lock(&self, state: bool) {
        self.network_lock.store(state, Ordering::Relaxed);
    }

    async fn get_delta_time(&self) -> Duration {
        let mut t = self.timer.write().await;
        let delta_time = t.elapsed();
        *t = Instant::now();
        delta_time
    }

    pub fn spawn_network_thread(&self) {
        let container = self.clone();
        log::info!(target: "network", "Spawning network thread...");

        std::thread::spawn(move || {
            let io_loop = tokio::runtime::Runtime::new().unwrap();
            io_loop.block_on(async move {
                let network = container.client_network.read().await;
                loop {
                    // Network will be processed only when there is no lock
                    if container.is_network_locked() {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    container.set_network_lock(true);

                    let success = network.step(container.get_delta_time().await).await;
                    if !success {
                        log::info!(target: "network", "Network thread exited;");
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            });
        });
        log::info!(target: "network", "Network thread spawned");
    }

    pub fn disconnect(&self) {
        let network = self.get_client();

        if network.is_connected() {
            log::info!(target: "network", "Disconnected from the server");
            network.disconnect();
        }
    }
}
