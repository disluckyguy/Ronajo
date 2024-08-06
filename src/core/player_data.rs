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

    pub fn start_session(&self, url: &str, player: &str) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        match player {
            "MPV" => {
                channel
                    .exec(
                        &format!(
                            "systemd-run --user --scope --unit=mpv-session --setenv=DISPLAY=:0 nohup setsid mpv {} --input-ipc-server={} --no-terminal > /dev/null 2>&1 &",
                            url,
                            SOCKET_PATH
                        )
                )?;
            }

            "VLC" => {
                channel
                    .exec(
                        &format!(
                            "systemd-run --user --scope --unit=vlc-session --setenv=DISPLAY=:0 nohup setsid vlc {} > /dev/null 2>&1 &",
                            url
                        )
                )?;

                let mut output = String::new();
                channel.read_to_string(&mut output).unwrap();

                if output.contains("command not found") {
                    channel
                        .exec(
                            &format!(
                                "systemd-run --user --scope --unit=vlc-session --setenv=DISPLAY=:0 nohup setsid flatpak run org.videolan.VLC {}  > /dev/null 2>&1 &",
                                url
                            )
                    )?;
                }
            }
            _  => unreachable!()
        }

        channel.close()?;
        Ok(())
    }

    pub fn quit(&self, player: &str) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        match player {
            "MPV" => {
                channel
                    .exec(&format!("echo 'quit' | socat - {}", SOCKET_PATH))?;
            }
            "VLC" => {
                channel
                    .exec("pkill vlc")?;
            }
            _ => unreachable!()
        }

        channel.close()?;
        Ok(())
    }

    pub fn screenshot(&self, player: &str) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;

        match player {
            "MPV" => {
                channel
                    .exec(&format!("echo 'quit' | socat - {}", SOCKET_PATH))?;
            }
            "VLC" => {
                return Err("Unsupported".into())
            }
            _ => unreachable!()
        }
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

    pub fn toggle_pause(&self, player: &str) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        match player {
            "MPV" => {
                channel
                    .exec(&format!("echo 'cycle pause' | socat - {}", SOCKET_PATH))?;
            }
            "VLC" => {
                channel
                    .exec("busctl --user call org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player PlayPause")?;
            }
            _ => unreachable!()
        }

        channel.close()?;
        Ok(())
    }

    pub fn set_paused(&self, paused: bool, player: &str) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        match player {
            "MPV" => {
                let value = if paused {
                    String::from("yes")
                } else {
                    String::from("no")
                };
                channel
                    .exec(&format!("echo 'set pause {}' | socat - {}", value, SOCKET_PATH))?;
            }
            "VLC" => {
                let value = if paused {
                    String::from("Pause")
                } else {
                    String::from("Play")
                };
                channel
                    .exec(&format!("busctl --user call org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player {}", value))?;
            }
            _ => unreachable!()
        }

        channel.close()?;
        Ok(())
    }

    pub fn set_volume(&self, volume: f64, player: &str) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;

        match player {
            "MPV" => {
                channel
                    .exec(&format!("echo 'set volume {}' | socat - {}", volume, SOCKET_PATH))?;
            }
            "VLC" => {
                channel
                    .exec(&format!("busctl --user set-property org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Volume d {}", volume / 100f64))?;
            }
            _ => unreachable!()
        }
        channel.close()?;
        Ok(())
    }

    pub fn set_rate(&self, rate: f64, player: &str) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        match player {
            "MPV" => {
                channel
                    .exec(&format!("echo 'set speed {}' | socat - {}", rate, SOCKET_PATH))?;
            }
            "VLC" => {
                channel
                    .exec(&format!("busctl --user set-property org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Rate d {}", rate))?;
            }
            _ => unreachable!()
        }
        channel.close()?;
        Ok(())
    }

    pub fn set_muted(&self, muted: bool, player: &str) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        match player {
            "MPV" => {
                if muted {
                    channel
                        .exec(&format!("echo 'set mute {}' | socat - {}", "yes", SOCKET_PATH))?;
                } else {
                    channel
                        .exec(&format!("echo 'set mute {}' | socat - {}", "no", SOCKET_PATH))?;
                }
            }
            "VLC" => {
                if muted {
                    channel
                        .exec(&format!("busctl --user set-property org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Volume d {}", 0))?;
                } else {
                    channel
                        .exec(&format!("busctl --user set-property org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Volume d {}", 50))?;
                }

            }
            _ => unreachable!()
        }

        channel.close()?;
        Ok(())
    }

    // pub fn get_volume(&self, player: &str) -> Result<(), Box<dyn Error>>{
    //     let session = self.connect()?;
    //     let mut channel = session.channel_session()?;
    //     channel
    //         .exec("systemd-run --user --scope --unit=mpv-session --setenv=DISPLAY=:0 nohup setsid mpv http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4 --fs--input-ipc-server=/tmp/mpvsocket > /dev/null 2>&1 &")?;
    //     channel.close()?;
    //     Ok(())
    // }

    // pub fn get_rate(&self, player: &str) -> Result<(), Box<dyn Error>>{
    //     let session = self.connect()?;
    //     let mut channel = session.channel_session()?;
    //     channel
    //         .exec("systemd-run --user --scope --unit=mpv-session --setenv=DISPLAY=:0 nohup setsid mpv http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4 --fs--input-ipc-server=/tmp/mpvsocket > /dev/null 2>&1 &")?;
    //     channel.close()?;
    //     Ok(())
    // }

    pub fn get_duration(&self, player: &str) -> Result<f64, Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        let duration = match player {
            "MPV" => {
                channel
                    .exec(&format!("echo '{}' | socat - {}", "{\"command\": [\"get_property\", \"duration\"]}", SOCKET_PATH))?;
                let mut output = String::new();
                channel.read_to_string(&mut output)?;
                let response: SocketResponse = serde_json::from_str(&output.trim())?;
                response.data
            }
            "VLC" => {
                channel
                    .exec("busctl --user get-property org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Metadata")?;

                let mut output = String::new();
                channel.read_to_string(&mut output)?;
                let output_words: Vec<&str> = output.split_whitespace().collect();
                let length_index = output_words.iter().position(|&r| r.contains("mpris:length")).expect("field not found");

                let duration: f64 = output_words[length_index + 2].parse()?;
                duration / 1000000f64
            }
            _ => unreachable!()
        };

        channel.close()?;

        Ok(duration)


    }

    pub fn get_position(&self, player: &str) -> Result<f64, Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        let position = match player {
            "MPV" => {
                channel
                    .exec(&format!("echo '{}' | socat - {}", "{\"command\": [\"get_property\", \"playback-time\"]}", SOCKET_PATH))?;
                let mut output = String::new();
                channel.read_to_string(&mut output)?;
                let response: SocketResponse = serde_json::from_str(&output.trim())?;
                response.data
            }
            "VLC" => {
                channel
                    .exec("busctl --user get-property org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Position")?;
                let mut output = String::new();
                channel.read_to_string(&mut output)?;
                let output_words: Vec<&str> = output.split_whitespace().collect();
                let position: f64 = output_words[1].parse()?;
                position / 1000000f64
            }
            _ => unreachable!()
        };

        channel.close()?;

        Ok(position)
    }

    pub fn seek_forward(&self, player: &str) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;

        match player {
            "MPV" => {
                channel
                    .exec(&format!("echo 'seek {}' | socat - {}", 10f64, SOCKET_PATH))?;
            }
            "VLC" => {
                channel
                    .exec(&format!("busctl --user call org.mpris.MediaPlayer2.vlc -- /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Seek x {}", 10 * 1000000))?;
            }
            _ => unreachable!()
        }

        channel.close()?;
        Ok(())
    }

    pub fn seek_backward(&self, player: &str) -> Result<(), Box<dyn Error>>{
        let session = self.connect()?;
        let mut channel = session.channel_session()?;
        match player {
            "MPV" => {
                channel
                    .exec(&format!("echo 'seek {}' | socat - {}", -10f64, SOCKET_PATH))?;
            }
            "VLC" => {
                channel
                    .exec(&format!("busctl --user call org.mpris.MediaPlayer2.vlc -- /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Seek x {}", -10 * 1000000))?;
            }
            _ => unreachable!()
        }
        channel.close()?;
        Ok(())
    }

    pub fn seek_to(&self, position: f64, player: &str) -> Result<(), Box<dyn Error>> {
        let session = self.connect()?;
        let mut channel = session.channel_session()?;


        match player {
            "MPV" => {
                channel
                    .exec(&format!("echo 'seek {} absolute' | socat - {}", position, SOCKET_PATH))?;
            }
            "VLC" => {
                channel
                    .exec("busctl --user get-property org.mpris.MediaPlayer2.vlc /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Metadata")?;


                let mut output = String::new();
                channel.read_to_string(&mut output)?;
                let output_words: Vec<&str> = output.split_whitespace().collect();
                let id_index = output_words.iter().position(|&r| r.contains("mpris:trackid"));
                let track_id: &str = if let Some(index) = id_index {
                    output_words[index + 2]
                } else {
                    "/"
                };

                channel.close()?;

                let mut channel = session.channel_session()?;

                channel
                    .exec(&format!("dbus-send --session --dest=org.mpris.MediaPlayer2.vlc --type=method_call /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.SetPosition objpath:{} int64:{}", track_id, position as i64 * 1000000))?;

                let mut output = String::new();
                channel.read_to_string(&mut output)?;
                println!("{}", output);

            }
            _ => unreachable!()
        }
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

