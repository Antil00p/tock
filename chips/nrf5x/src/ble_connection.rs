use ble_advertising_driver::LLData;
use core::fmt;
use ble_advertising_hil::RadioChannel;

// pub struct LLData {
//     pub aa: [u8; 4],
//     pub crc_init: [u8; 3],
//     win_size: u8,
//     win_offset: u16,
//     interval: u16,
//     latency: u16,
//     timeout: u16,
//     chm: [u8; 5],
//     hop_and_sca: u8 // hops 5 bits, sca 3 bits
// }

const NUMBER_CHANNELS: usize = 40;
const NUMBER_DATA_CHANNELS: usize = NUMBER_CHANNELS - 3;

type ChannelMap = [u8; NUMBER_CHANNELS];

#[derive(Copy, Clone)]
pub struct ConnectionData {
	last_unmapped_channel: u8,
	channels: ChannelMap,
	conn_event_counter: u16,
	hop_increment: u8,
	number_used_channels: u8,
}

impl PartialEq for ConnectionData {
    fn eq(&self, other: &ConnectionData) -> bool {
        self.last_unmapped_channel == other.last_unmapped_channel
    }
}

impl fmt::Debug for ConnectionData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ConnectionData {{ last_unmapped_channel: {}, conn_event_counter: {}, hop_increment: {}, number_used_channels: {} }}",
            self.last_unmapped_channel,
            self.conn_event_counter,
            self.hop_increment,
            self.number_used_channels
        )
    }
}


impl ConnectionData {
	pub fn new(lldata: &LLData) -> ConnectionData {

		let (channels, number_used_channels) = ConnectionData::expand_channel_map(lldata.chm);

	    ConnectionData {
	    	last_unmapped_channel: 0,
	    	channels,
	    	number_used_channels,
			hop_increment: lldata.hop_and_sca & 0b11111,
            conn_event_counter: 0
	    }
	}

	pub fn update_lldata(&mut self, lldata: LLData) {
		let (channels, number_used_channels) = ConnectionData::expand_channel_map(lldata.chm);

		self.channels = channels;
		self.number_used_channels = number_used_channels;
	}

	fn expand_channel_map(chm: [u8; 5]) -> (ChannelMap, u8) {
		let mut channels: ChannelMap = [0; NUMBER_CHANNELS];

		let mut number_used_channels = 0;

	    for i in 0..chm.len() {
	        let mut byte = chm[i];

	        for j in 0..8 {
	            let bit = (byte as u8) & 1;

	            if bit == 1 {
	                number_used_channels += 1;
	            }

	            channels[(i * 8) + j] = bit;
	            byte = byte >> 1;
	        }
	    }

        (channels, number_used_channels)
	}

	pub fn next_channel(&mut self) -> RadioChannel {
	    let unmapped_channel = (self.last_unmapped_channel + self.hop_increment) % (NUMBER_DATA_CHANNELS as u8);
	    let used = self.channels[unmapped_channel as usize] == 1;

        self.last_unmapped_channel = unmapped_channel;

	    if used {
            RadioChannel::from_channel_index(unmapped_channel).unwrap()
	    } else {

	        let mut table: ChannelMap = [0; NUMBER_CHANNELS];
	        let remapping_index = unmapped_channel % self.number_used_channels;

	        let mut idx = 0;

	        for i in 0..self.channels.len() {
	            if self.channels[i] == 1 {
	                table[idx] = i as u8;
	                idx += 1;
	            }
	        }

            RadioChannel::from_channel_index(table[remapping_index as usize]).unwrap()
	    }
	}
}


