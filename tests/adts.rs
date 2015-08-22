#![allow(dead_code)]

#[macro_use]
extern crate nom;

use nom::{HexDisplay,Needed,IResult,FileProducer,be_u16,be_u32,be_u64,be_f32};
use nom::{Consumer,ConsumerState};
use nom::IResult::*;
use nom::Err::*;

use std::str;
use std::io::SeekFrom;

fn adts_header(input: &[u8]) -> IResult<&[u8], &[u8]> {
  match be_u32(input) {
    Done(i, offset) => {
      let sz: usize = offset as usize;
      if i.len() >= sz - 4 {
        return Done(&i[(sz-4)..], &i[0..(sz-4)])
      } else {
        return Incomplete(Needed::Size(offset as usize + 4))
      }
    }
    Error(e)      => Error(e),
    Incomplete(e) => Incomplete(e)
  }
}


#[derive(Debug)]
pub struct ADTSHeader {
  // Header starts with 12-bit syncword, all 1s.
  version: u8,
  layer: u8,
  protection_absent: bool,
  profile: u8,
  frequency_index: u8,
  private: bool,
  channel_config: u8,
  original: bool,
  home: bool,
  copyright: bool,
  copyright_start: bool,
  frame_length: usize,
  buffer_fullness: usize,
  frame_count: u8,
  crc: Option<u16>,
}

enum ADTSState {
    Header,
    Body,
}

pub struct ADTSConsumer {
  state: ADTSState,
  bytes: usize
}

impl ADTSConsumer {
  fn new() -> ADTSConsumer {
    ADTSConsumer { state: ADTSState::Header, bytes: 0 }
  }

  fn consume_header(&mut self, input: &[u8]) -> ConsumerState {
    println!("\nparsing ADTS header:\n{}", input.to_hex(8));
    match adts_header(input) {
      Done(i, offset) => {
        match offset {
            bytes => println!("offset {} bytes", bytes.len()),
        };
        // current producer position is at input.len()
        // I consumed input.len() - i.len() = header.frame_length.
        // I want to advance from header.length()
        // offset to my new position: -input.len() + (input.len() - i.len()) + header.len() == header.len() - i.len()
        return ConsumerState::Seek(input.len() - i.len(), SeekFrom::Current((offset.len() - i.len() as usize - 8) as i64), 100);
      },
      Error(a) => {
        println!("adts parsing error: {:?}", a);
        assert!(false);
        return ConsumerState::ConsumerError(0);
      },
      Incomplete(_) => {
        // FIXME: incomplete should send the required size
        println!("adts incomplete -> await: {}", input.len());
        return ConsumerState::Await(0, input.len() + 100);
      }
    }
  }

  fn consume_body(&mut self, input: &[u8]) -> ConsumerState {
      println!("\nparsing ATDS body");
      Done()
  }
}


impl Consumer for ADTSConsumer {
  fn consume(&mut self, input: &[u8]) -> ConsumerState {
    match self.state {
      ADTSState::Header => {
        self.consume_header(input)
      },
      ADTSState::Body => {
        self.consume_body(input)
      },
    }
  }

  fn failed(&mut self, error_code: u32) {
    println!("failed with error code: {}", error_code);
  }

  fn end(&mut self) {
    println!("finish!");
  }
}

#[allow(unused_must_use)]
fn explore_adts_file(filename: &str) {
  FileProducer::new(filename, 400).map(|producer: FileProducer| {
    println!("file producer created for {}", filename);
    let mut p = producer;
    let mut c = ADTSConsumer{state: ADTSState::Header, bytes: 0};
    c.run(&mut p);

    //assert!(false);
  });
}

/*
#[test]
fn file_test() {
  explore_adts_file("./small.aac");
}


#[test]
fn bunny_test() {
  //explore_adts_file("bigbuckbunny.aac");
}
*/
