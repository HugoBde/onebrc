use std::collections::HashMap;
use std::env::args;
use std::io::BufRead;

type Info = (f64, f64, f64, u32);

fn main() {
  let args: Vec<_> = args().collect();

  let path = if args.len() == 2 {
    args[1].as_str()
  } else {
    "data/measurements.txt"
  };

  let f = std::fs::File::open(path).unwrap();
  let f = std::io::BufReader::new(f);

  let mut data = HashMap::<String, Info>::new();

  for l in f.lines() {
    let l = l.unwrap();
    let mut a = l.split(';');
    let station = a.next().unwrap();
    let temp: f64 = a.next().unwrap().parse().unwrap();

    match data.get_mut(station) {
      Some(data) => {
        data.0 = data.0.min(temp);
        data.1 = data.1.max(temp);
        data.2 += temp;
        data.3 += 1;
      }
      None => {
        data.insert(station.to_string(), (temp, temp, temp, 1));
      }
    };
  }

  let mut data: Vec<_> = data
    .into_iter()
    .map(|(key, value)| {
      let mean = value.2 / (value.3 as f64);
      (key, (value.0, value.1, mean))
    })
    .collect();

  data.sort_by(|a, b| a.0.cmp(&b.0));

  print!("{{");
  for entry in data {
    print!(
      "{}={:.1}/{:.1}/{:.1}",
      entry.0, entry.1.0, entry.1.2, entry.1.1
    );
  }
  println!("}}");
}
