#![warn(rust_2018_idioms)]

use libopus::decoder::Decoder as OpusDecoder;
use rtp_rs::*;

use std::io;
use std::fs::File;
use std::net::SocketAddr;
use wav;

use tokio::net::UdpSocket;
use structopt::StructOpt;

use libopus::decoder::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "decoder", about = "Opus decoding")]
struct DecodingOpts {
    /// Sampling rate, in Hz
    #[structopt(default_value = "48000")]
    sampling_rate: usize,
    /// Channels, either 1 or 2
    #[structopt(default_value = "1")]
    channels: usize,
    /// Number of seconds to decode
    #[structopt(default_value = "10")]
    seconds: i32,
}
trait Decode {
    fn get_decoder(&self) -> Option<OpusDecoder>;
}

impl Decode for DecodingOpts {
    fn get_decoder(&self) -> Option<OpusDecoder> {
        if self.channels > 2 {
            unimplemented!("Multichannel support missing");
        }

        let coupled_streams = if self.channels > 1 { 1 } else { 0 };
        Decoder::create(self.sampling_rate, self.channels, 1, coupled_streams, &[0u8, 1u8]).ok()
    }
}
pub struct Server {
    pub socket: UdpSocket,
    pub buf: Vec<u8>,
    pub to_send: Option<(usize, SocketAddr)>,
}

impl Server {
    pub async fn run(self) -> Result<(), io::Error> {
        let dec_opt = DecodingOpts::from_args();

        let mut dec = dec_opt.get_decoder().unwrap();
    
        let mut _out_f = File::create("out.wav").unwrap();

        let max_packet: usize = 1500;
        let max_frame: usize = 48000 * 2;
        let max_frame_samples: usize = max_frame * dec_opt.channels as usize;

        let mut pkt = Vec::with_capacity(max_packet);
        let mut samples = Vec::with_capacity(max_frame_samples);
        let mut result_buffer:Vec<i16> = Vec::new();

        pkt.resize(max_packet, 0u8);
        samples.resize(max_frame_samples, 0i16);

        let Server {
            mut socket,
            mut buf,
            mut to_send,
        } = self;
        let mut pkt_counter = 0;

        loop {
            pkt_counter += 1;
            if let Some((_size , _)) = to_send {
                let len = buf.iter().filter(|&n| *n != 0).count() + 1;
                if let Ok(rtp) = RtpReader::new(&buf[..len]) {
                    let payload = rtp.payload();
                    dec.decode(payload, &mut samples[..], false).unwrap();
                }

                let short_samples = &samples.iter().cloned().filter(|&p| p != 0i16).collect::<Vec<i16>>();
                result_buffer.extend(short_samples);

                // 1000 samples just for test a piece of file
                if pkt_counter == 1000 {
                    let header = wav::Header::new(1, 2, 16000, 16);
                    wav::write_wav(header, wav::BitDepth::Sixteen(result_buffer.clone()), std::path::Path::new("data/output.wav")).unwrap();
                }

            }
            to_send = Some(socket.recv_from(&mut buf).await?);
        }
    }
}