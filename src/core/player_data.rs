use std::error::Error;
use serde::{Serialize, Deserialize};
use ssh2::Session;
use std::net::TcpStream;
use std::io::Read;

static SOCKET_PATH: &str = "/tmp/playersocket";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayerData {
    pub name: String,
    pub address: String,
    pub username: String,
    #[serde(skip)]
    pub password: Option<String>,
    pub use_key: bool
}

impl PlayerData {

    pub fn new(name: String, address: String, username: String, password: Option<String>, use_key: bool) -> PlayerData {
        PlayerData {
            name,
            address,
            username,
            password,
            use_key
        }
    }

    fn connect(&self) -> Result<Session, Box<dyn Error>> {
        let tcp = TcpStream::connect(format!("{}:22", &self.address))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        match self.password {
            Some(ref password) => session.userauth_password(&self.username, &password)?,
            None => session.userauth_agent(&self.username)?,
        };
        Ok(session)
    }

    pub fn validate(&self)  -> Result<(), Box<dyn Error>> {
        if let Err(err) = self.connect() {
            println!("error");
            return Err(err);
        }
        Ok(())
    }

    pub fn start_session(&self, url: &str) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(
            &format!(
                "systemd-run --user --scope --unit=mpv-session --setenv=DISPLAY=:0 nohup setsid mpv {} --input-ipc-server={} --no-terminal > /dev/null 2>&1 &",
                 url,
                 SOCKET_PATH
                )
            )?;
        channel.close()?;
        Ok(())
    }

    pub fn quit(&self) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo 'quit' | socat - {}", SOCKET_PATH))?;
        channel.close()?;
        Ok(())
    }

    pub fn screenshot(&self) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo 'screenshot' | socat - {}", SOCKET_PATH))?;
        channel.close()?;

//         let (mut remote_file, stat) = sess.scp_recv(Path::new("remote")).unwrap();
//         let mut contents = Vec::new();
//         remote_file.read_to_end(&mut contents).unwrap();

// Close the channel and wait for the whole content to be tranferred
//         remote_file.send_eof().unwrap();
//         remote_file.wait_eof().unwrap();
//         remote_file.close().unwrap();
//         remote_file.wait_close().unwrap();
        Ok(())
    }

    pub fn toggle_pause(&self) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo 'cycle pause' | socat - {}", SOCKET_PATH))?;
        channel.close()?;
        Ok(())
    }

    pub fn set_paused(&self, paused: bool) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        let value = if paused {
            String::from("yes")
        } else {
            String::from("no")
        };
        channel
            .exec(&format!("echo 'set pause {}' | socat - {}", value, SOCKET_PATH))?;
        channel.close()?;
        Ok(())
    }

    pub fn set_volume(&self, volume: f64) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo 'set volume {}' | socat - {}", volume, SOCKET_PATH))?;
        channel.close()?;
        Ok(())
    }

    pub fn set_rate(&self, rate: f64) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo 'set speed {}' | socat - {}", rate, SOCKET_PATH))?;
        channel.close()?;
        Ok(())
    }

    pub fn set_muted(&self, muted: bool) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        if muted {
            channel
                .exec(&format!("echo 'set mute {}' | socat - {}", "yes", SOCKET_PATH))?;
        } else {
            channel
                .exec(&format!("echo 'set mute {}' | socat - {}", "no", SOCKET_PATH))?;
        }

        channel.close()?;
        Ok(())
    }

    pub fn get_volume(&self) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec("systemd-run --user --scope --unit=mpv-session --setenv=DISPLAY=:0 nohup setsid mpv http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4 --fs--input-ipc-server=/tmp/mpvsocket > /dev/null 2>&1 &")?;
        channel.close()?;
        Ok(())
    }

    pub fn get_rate(&self) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec("systemd-run --user --scope --unit=mpv-session --setenv=DISPLAY=:0 nohup setsid mpv http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4 --fs--input-ipc-server=/tmp/mpvsocket > /dev/null 2>&1 &")?;
        channel.close()?;
        Ok(())
    }

    pub fn get_duration(&self) -> Result<f64, Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo '{}' | socat - {}", "{\"command\": [\"get_property\", \"duration\"]}", SOCKET_PATH))?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        let response: SocketResponse = serde_json::from_str(&output.trim())?;
        channel.close()?;

        Ok(response.data)
    }

    pub fn get_position(&self) -> Result<f64, Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo '{}' | socat - {}", "{\"command\": [\"get_property\", \"playback-time\"]}", SOCKET_PATH))?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        let response: SocketResponse = serde_json::from_str(&output.trim())?;
        channel.close()?;

        Ok(response.data)
    }

    pub fn seek_forward(&self) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo 'seek {}' | socat - {}", 10f64, SOCKET_PATH))?;
        channel.close()?;
        Ok(())
    }

    pub fn seek_backward(&self) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo 'seek {}' | socat - {}", -10f64, SOCKET_PATH))?;
        channel.close()?;
        Ok(())
    }

    pub fn seek_to(&self, position: f64) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        channel
            .exec(&format!("echo 'seek {} absolute' | socat - {}", position, SOCKET_PATH))?;
        channel.close()?;
        Ok(())
    }

}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct SocketResponse {
    pub data: f64,
    pub request_id: i32,
    pub error: String
}

