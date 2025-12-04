#![feature(slice_split_once)]
#![feature(portable_simd)]

use std::env::args;
use std::ptr::read_unaligned;
use std::simd::Simd;
use std::simd::cmp::SimdPartialEq;

use memmap2::Mmap;
use rustc_hash::FxHashMap;

type Info = (i64, i64, i64, u32);

const SIMD_LANES: usize = 16;

fn main() {
  let args: Vec<_> = args().collect();

  let path = if args.len() == 2 {
    args[1].as_str()
  } else {
    "data/measurements.txt"
  };

  let f = std::fs::File::open(path).unwrap();
  let mmap = unsafe { Mmap::map(&f).unwrap() };

  let mut data = FxHashMap::<&[u8], Info>::default();

  for l in mmap.split(|c| *c == b'\n') {
    if l.is_empty() {
      break;
    }

    let (station, temp) = l.rsplit_once(|c| *c == b';').unwrap();
    let temp = temp_parser(temp);

    match data.get_mut(station) {
      Some(data) => {
        data.0 = data.0.min(temp);
        data.1 = data.1.max(temp);
        data.2 += temp;
        data.3 += 1;
      }
      None => {
        data.insert(station, (temp, temp, temp, 1));
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

fn rsplit_once<'l>(line: &'l [u8]) -> (&'l [u8], &'l [u8]) {
  let mut offset = 0;

  while offset + SIMD_LANES < line.len() {
    let s = unsafe { read_unaligned(line[offset..].as_ptr() as *const Simd<u8, SIMD_LANES>) };
    if let Some(index) = simd_find(s) {
      return (&line[..offset + index], &line[offset + index + 1..]);
    }
    offset += SIMD_LANES;
  }

  let s = Simd::<u8, SIMD_LANES>::load_or_default(&line[offset..]);

  let index = simd_find(s).unwrap();
  return (&line[..offset + index], &line[offset + index + 1..]);
}

// const NEW_LINE_SIMD: Simd<u8, SIMD_LANES> = Simd::<u8,
// SIMD_LANES>::splat(b'\n');
const SEMI_COLON_SIMD: Simd<u8, SIMD_LANES> = Simd::<u8, SIMD_LANES>::splat(b';');

fn simd_find(s: Simd<u8, SIMD_LANES>) -> Option<usize> {
  SEMI_COLON_SIMD.simd_eq(s).first_set()
}

#[rustfmt::skip]
fn temp_parser(bytes: &[u8]) -> i64 {
  match (bytes[0], bytes.len()) {
    (b'-', 5) => {-((bytes[1] - b'0') as i64 * 100 + (bytes[2] - b'0') as i64 * 10 + (bytes[4] - b'0') as i64)},
    (b'-', 4) => {-(                                 (bytes[1] - b'0') as i64 * 10 + (bytes[3] - b'0') as i64)},
    (_, 4) =>    {  (bytes[0] - b'0') as i64 * 100 + (bytes[1] - b'0') as i64 * 10 + (bytes[3] - b'0') as i64},
    (_, 3) =>    {                                   (bytes[0] - b'0') as i64 * 10 + (bytes[2] - b'0') as i64},
    (_, _) => unreachable!("not possible")
  }
}

#[cfg(test)]
mod tests {
  use crate::{rsplit_once, temp_parser};

  #[test]
  fn line_splitter() {
    let lines: Vec<_> = vec![
      "Pontianak;40.9",
      "Edmonton;1.7",
      "Suva;32.5",
      "Boston;16.3",
      "Las Vegas;20.6",
      "Prague;0.7",
      "Ndola;22.8",
      "Lyon;25.6",
      "Antsiranana;19.6",
      "Ashgabat;19.1",
      "Valencia;13.0",
      "Damascus;17.1",
      "Boise;13.4",
      "Nouakchott;20.8",
      "Lahore;8.8",
      "Belgrade;22.3",
      "Yerevan;13.6",
      "Ségou;43.4",
      "Andorra la Vella;7.5",
    ]
    .iter()
    .map(|l| l.as_bytes())
    .collect();

    let expect: Vec<_> = lines
      .iter()
      .map(|l| l.rsplit_once(|c| *c == b';').unwrap())
      .collect();

    let res: Vec<_> = lines.iter().map(|l| rsplit_once(l)).collect();

    assert_eq!(res, expect);
  }

  #[test]
  fn temperature_parsing() {
    let data = vec![
      "-58.5", "22.4", "-19.4", "-7.6", "44.7", "45.6", "-38.4", "97.3", "-74.3", "33.8", "42.6",
      "-81.6", "-69.5", "34.5", "-4.9", "-51.1", "-17.6", "71.1", "40.4", "-71.5", "75.7", "85.3",
    ];

    let expect = vec![
      -585, 224, -194, -76, 447, 456, -384, 973, -743, 338, 426, -816, -695, 345, -49, -511, -176,
      711, 404, -715, 757, 853,
    ];

    let out: Vec<i64> = data.iter().map(|s| temp_parser(s.as_bytes())).collect();
    assert_eq!(out, expect);
  }
}
