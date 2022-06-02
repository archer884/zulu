use std::{borrow::Cow, fmt, str::FromStr};

use chrono::{DateTime, Local, Timelike, Utc};
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    time: Option<Time>,
    am_pm: Option<Meridian>,

    /// optional time format string; applied to output
    #[clap(short, long)]
    time_format: Option<String>,
}

impl Args {
    fn zulu(&self) -> DateTime<Utc> {
        let date = Local::now().date();
        let time = self.time.unwrap_or_else(|| Local::now().into());
        let hours = if time.hours < 12 && self.meridian().is_pm() {
            time.hours + 12
        } else {
            time.hours
        };
        date.and_hms(hours.into(), time.minutes.into(), 0).into()
    }

    fn meridian(&self) -> Meridian {
        self.am_pm.unwrap_or_else(|| {
            if Local::now().hour() < 12 {
                Meridian::AM
            } else {
                Meridian::PM
            }
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct Time {
    hours: u8,
    minutes: u8,
}

impl From<DateTime<Local>> for Time {
    fn from(time: DateTime<Local>) -> Self {
        Time {
            hours: time.hour12().1 as u8,
            minutes: time.minute() as u8,
        }
    }
}

impl FromStr for Time {
    type Err = ParseHoursMinutesErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let result = Time {
            hours: parts
                .next()
                .ok_or_else(|| ParseHoursMinutesErr("missing hours".into()))?
                .parse()
                .map_err(|e| ParseHoursMinutesErr(format!("unable to parse hours: {e}").into()))?,
            minutes: parts
                .next()
                .ok_or_else(|| ParseHoursMinutesErr("missing minutes".into()))?
                .parse()
                .map_err(|e| {
                    ParseHoursMinutesErr(format!("unable to parse minutes: {e}").into())
                })?,
        };

        if parts.next().is_some() {
            return Err(ParseHoursMinutesErr("bad time format".into()));
        }

        Ok(result)
    }
}

#[derive(Clone, Copy, Debug)]
enum Meridian {
    AM,
    PM,
}

impl Meridian {
    #[inline]
    fn is_pm(self) -> bool {
        match self {
            Meridian::AM => false,
            Meridian::PM => true,
        }
    }
}

impl FromStr for Meridian {
    type Err = MeridianErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AM" | "am" => Ok(Meridian::AM),
            "PM" | "pm" => Ok(Meridian::PM),
            unknown => Err(MeridianErr(unknown.into())),
        }
    }
}

#[derive(Debug)]
struct MeridianErr(String);

impl fmt::Display for MeridianErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown am/pm marker: {}", self.0)
    }
}

impl std::error::Error for MeridianErr {}

#[derive(Debug)]
struct ParseHoursMinutesErr(Cow<'static, str>);

impl fmt::Display for ParseHoursMinutesErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseHoursMinutesErr {}

fn main() {
    run(&Args::parse());
}

fn run(args: &Args) {
    let zulu = args.zulu();
    let formatted_time = match &args.time_format {
        Some(fmt) => zulu.format(fmt),
        None => zulu.format("%R"),
    };
    println!("{formatted_time}");
}
