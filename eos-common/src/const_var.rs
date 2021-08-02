pub const APP_VERSION: u32 = 0;
pub const SERVER_ADDR: &str = "127.0.0.1:8484";
pub const SERVER_PORT: &str = "0.0.0.0:8484";

pub const MAIN_LOOP_DURATION: std::time::Duration = std::time::Duration::from_millis(100);

/// Number of sectors in SpaceGrid on the x axis.
pub const X_SECTOR: i32 = 12;
/// Number of sectors in SpaceGrid on the y axis.
pub const Y_SECTOR: i32 = 9;
pub const NUM_SECTOR: usize = (X_SECTOR * Y_SECTOR) as usize;

/// Sector go from -SECTOR_SIZE to SECTOR_SIZE. 0 is the center.
pub const SECTOR_HALF_SIZE: f32 = 8192.0;
/// Total sector size.
pub const SECTOR_SIZE: f32 = SECTOR_HALF_SIZE * 2.0;

/// Size allocated for each connection to write and unparsed.
pub const CONNECTION_BUF_SIZE: usize = 16384;
pub const CLIENT_LOCAL_PACKET_BUFFER: usize = 8;

/// How long a socket can spend in the login step without answering.
pub const MAX_LOGIN_WAIT: std::time::Duration = std::time::Duration::from_secs(5);
/// Max new Connection the listening loop will accept before pausing.
pub const LISTENING_BUFFER: usize = 16;
/// Min duration a banned address will have to wait after being temp ban.
pub const LISTENING_TEMP_BAN_DURATION: std::time::Duration = std::time::Duration::from_secs(10);
/// Max LoginSuccess login loop will send before pausing.
pub const LOGIN_SUCCESS_BUFFER: usize = 32;
/// Min duration a banned address will have to wait after being temp ban.
pub const LOGIN_TEMP_BAN_DURATION: std::time::Duration = std::time::Duration::from_secs(300);
/// Max number of simultanious login processed.
pub const LOGIN_PROGRESS_BUFFER: usize = 64;
/// How long a successful login will prevent anyone trying to login with the same ClientId.
pub const RECENTLY_REMOVED_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
