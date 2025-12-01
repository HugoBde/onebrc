#![feature(slice_split_once)]

use std::collections::HashMap;
use std::env::args;
use std::io::BufRead;

type Info = (i64, i64, i64, u32);

fn main() {
  let args: Vec<_> = args().collect();

  let path = if args.len() == 2 {
    args[1].as_str()
  } else {
    "data/measurements.txt"
  };

  let f = std::fs::File::open(path).unwrap();
  let f = std::io::BufReader::new(f);

  let mut data = HashMap::<Vec<u8>, Info>::new();

  for l in f.split(b'\n') {
    let l = l.unwrap();
    let (station, temp) = l.split_once(|c| *c == b';').unwrap();
    let temp = my_own_i64_parser(temp);

    match data.get_mut(station) {
      Some(data) => {
        data.0 = data.0.min(temp);
        data.1 = data.1.max(temp);
        data.2 += temp;
        data.3 += 1;
      }
      None => {
        data.insert(station.to_vec(), (temp, temp, temp, 1));
      }
    };
  }

  let mut data: Vec<_> = data
    .iter()
    .map(|(key, value)| {
      let min = (value.0 as f64) / 10.0;
      let max = (value.1 as f64) / 10.0;
      let mean = (value.2 as f64) / (value.3 as f64 * 10.0);
      (str::from_utf8(key).unwrap(), (min, max, mean))
    })
    .collect();

  data.sort_by(|a, b| a.0.cmp(&b.0));

  print!("{{");
  for (station, info) in data {
    print!("{}={:.1}/{:.1}/{:.1}, ", station, info.0, info.2, info.1);
  }

  println!("}}");
}

fn my_own_i64_parser(bytes: &[u8]) -> i64 {
  if bytes[0] == b'-' {
    parse_neg(&bytes[1..])
  } else {
    parse_pos(bytes)
  }
}

fn parse_neg(bytes: &[u8]) -> i64 {
  let mut out = 0i64;
  for byte in bytes {
    if *byte == b'.' {
      continue;
    }
    out *= 10;
    out -= (byte - b'0') as i64;
  }

  return out;
}

fn parse_pos(bytes: &[u8]) -> i64 {
  let mut out = 0i64;
  for byte in bytes {
    if *byte == b'.' {
      continue;
    }
    out *= 10;
    out += (byte - b'0') as i64;
  }

  return out;
}

#[cfg(test)]
mod tests {
  use crate::my_own_i64_parser;

  #[test]
  fn i64_parser() {
    let data = vec![
      "-58.5", "22.4", "-19.4", "-7.6", "44.7", "45.6", "-38.4", "97.3", "-74.3", "33.8", "42.6",
      "-81.6", "-69.5", "34.5", "-4.9", "-51.1", "-17.6", "71.1", "40.4", "-71.5", "75.7", "85.3",
    ];

    let expect = vec![
      -585, 224, -194, -76, 447, 456, -384, 973, -743, 338, 426, -816, -695, 345, -49, -511, -176,
      711, 404, -715, 757, 853,
    ];

    let out: Vec<i64> = data
      .iter()
      .map(|s| my_own_i64_parser(s.as_bytes()))
      .collect();
    assert_eq!(out, expect);
  }
}
