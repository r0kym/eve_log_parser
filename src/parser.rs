/*!
This module will do the parsing of eve logs.

For now it's needed to have a specific synthax for the logs to be readable.
More work will be needed to make the parser universal.
*/

use chrono::{DateTime, NaiveDateTime, Utc};
use log::{debug, error, info};
use regex::{Captures, Regex};

use crate::models::*;

const HEADER_REGEX: &str = r"(?m)^------------------------------------------------------------$
^  Gamelog$
^  Listener: (?P<name>.+)$
^  Session Started: (?<timestamp>\d{4}.\d{2}.\d{2} \d{2}:\d{2}:\d{2})$
^------------------------------------------------------------$";

const DAMAGE_REXEX: &str = r"(?i)\[ (?P<timestamp>\d{4}.\d{2}.\d{2} \d{2}:\d{2}:\d{2}) \] \(combat\) <color=0x[0-9a-f]{8}><b>(?P<damage>\d+)</b> <color=0x[0-9a-f]{8}><font size=\d+>(?P<destination>(to|from))</font> <b><color=0x[0-9a-f]{8}>(?P<pilot>.+)\[(?P<ticker>.+)\]\((?P<shiptype>.+)\)</b><font size=\d+><color=0x[0-9a-f]{8}> - (?P<weapon>.+) - ((Smashes)|(Penetrates)|(Hits)|(Glances Off)|(Grazes))\n";

const LOGI_REGEX: &str = r"(?i)^\[ (?P<timestamp>\d{4}.\d{2}.\d{2} \d{2}:\d{2}:\d{2}) \] \(combat\) <color=0x[0-9a-f]{8}><b>(?P<damage>\d+)</b><color=0x[0-9a-f]{8}><font size=\d+> remote ((armor)|(shield)|(hull)) .+ (?P<destination>(by|to)) </font><b><color=0x[0-9a-f]{8}><font size=\d+><color=0x[0-9a-f]{8}> <b>(?P<shiptype>.+)</b></color></font><color=0x[0-9a-f]{8}> \[(?P<pilot>.+)\]<color=0x[0-9a-f]{8}><b> -</color> </b><color=0x[0-9a-f]{8}><font size=\d+> - (?P<reptype>.+)</font>\n";

/// Read any log line from eve and creates an appropriate log if possible
///
/// # Example:
/// ```
/// use chrono::{TimeZone, Utc};
/// use eve_log_parser::models::{Log, DamageLog, Destination};
/// use eve_log_parser::parse_log_line;
///
///
/// let log: String = "[ 2024.07.02 20:31:28 ] (combat) <color=0xff00ffff><b>200</b> <color=0x77ffffff><font size=10>to</font> <b><color=0xffffffff>Hornet EC-300[-15.0](Hornet EC-300)</b><font size=10><color=0x77ffffff> - Draclira's Modified Tachyon Beam Laser - Penetrates\n".to_string();        
/// let expected_output = Log::Damage(DamageLog::new(
///     Utc.with_ymd_and_hms(2024, 7, 2, 20, 31, 28).unwrap(),
///     200,
///     "Hornet EC-300".to_string(),
///     "Hornet EC-300".to_string(),
///     "Draclira's Modified Tachyon Beam Laser".to_string(),
///     Destination::Dealing,
/// ));
///
/// let output = parse_log_line(&log).unwrap();
///
/// assert_eq!(expected_output, output);
/// ```
pub fn parse_log_line(text: &String) -> Option<Log> {
    info!("Parsing {}", text);
    let damage_re: Regex = Regex::new(DAMAGE_REXEX).unwrap();
    let logi_re: Regex = Regex::new(LOGI_REGEX).unwrap();

    if let Some(capture) = damage_re.captures(&text) {
        debug!("Damage log recognized");
        make_damage_log_from_capture(&capture)
    } else if let Some(capture) = logi_re.captures(&text) {
        debug!("Logi log recognized");
        make_logi_log_from_capture(&capture)
    } else {
        debug!("Log type not recognized");
        None
    }
}

/// Extracts the character name and the logfile's beginning from its header
pub fn parse_log_header(header: String) -> Option<(String, DateTime<Utc>)> {
    info!("Parsing header {}", header);
    let header_re = Regex::new(HEADER_REGEX).unwrap();

    if let Some(capture) = header_re.captures(header.as_str()) {
        debug!("Recognized the header");
        let log_beginning = parse_datetime(&capture["timestamp"]);
        let character_name = capture["name"].to_string();
        Some((character_name, log_beginning))
    } else {
        debug!("Didn't recognize the header format");
        None
    }
}

/// Takes the caputre of a regex and tries to create a Damage log out of it
fn make_damage_log_from_capture(capture: &Captures) -> Option<Log> {
    if let Ok(damage) = capture["damage"].parse::<isize>() {
        let destination = match capture["destination"].as_ref() {
            "to" => Destination::Dealing,
            "from" => Destination::Receiving,
            _ => panic!(
                "Unexpected token received - {}",
                capture["destination"].to_string()
            ),
        };
        Some(Log::Damage(DamageLog::new(
            parse_datetime(&capture["timestamp"]),
            damage,
            capture["pilot"].to_string(),
            capture["shiptype"].to_string(),
            capture["weapon"].to_string(),
            destination,
        )))
    } else {
        error!(
            "Couldn't parse the damage - {}",
            capture["damage"].to_string()
        );
        None
    }
}

/// Takes the caputre of a regex and tries to create a Logi log out of it
fn make_logi_log_from_capture(capture: &Captures) -> Option<Log> {
    if let Ok(amount) = capture["damage"].parse::<isize>() {
        let destination = match capture["destination"].as_ref() {
            "by" => Destination::Receiving,
            "to" => Destination::Dealing,
            _ => panic!(
                "Unexpected token received - {}",
                capture["destination"].to_string()
            ),
        };
        Some(Log::Logi(LogiLog::new(
            parse_datetime(&capture["timestamp"]),
            amount,
            capture["pilot"].to_string(),
            capture["shiptype"].to_string(),
            capture["reptype"].to_string(),
            destination,
        )))
    } else {
        error!(
            "Couldn't parse the amount - {}",
            capture["amount"].to_string()
        );
        None
    }
}

/// Recovers the datetime value from
fn parse_datetime(string: &str) -> DateTime<Utc> {
    let naive_datetime = NaiveDateTime::parse_from_str(string, "%Y.%m.%d %H:%M:%S").unwrap();
    DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::parser::{Destination, LogiLog};

    use super::{parse_log_header, parse_log_line, DamageLog, Log};

    #[test]
    fn test_basic_damage_logs() {
        let log_string1 = "[ 2024.07.02 20:31:28 ] (combat) <color=0xff00ffff><b>200</b> <color=0x77ffffff><font size=10>to</font> <b><color=0xffffffff>Hornet EC-300[-15.0](Hornet EC-300)</b><font size=10><color=0x77ffffff> - Draclira's Modified Tachyon Beam Laser - Penetrates\n".to_string();
        let log_string2 = "[ 2024.06.25 15:20:01 ] (combat) <color=0xffcc0000><b>375</b> <color=0x77ffffff><font size=10>from</font> <b><color=0xffffffff>Tek'wka Rokym[WH.SQ](Paladin)</b><font size=10><color=0x77ffffff> - Imperial Navy Large EMP Smartbomb - Hits\n".to_string();
        let log_string3 = "[ 2024.07.02 19:42:05 ] (combat) <color=0xff00ffff><b>153</b> <color=0x77ffffff><font size=10>to</font> <b><color=0xffffffff>Kilyavi Alaailaa[-15.0](Capsule)</b><font size=10><color=0x77ffffff> - Medium Vorton Projector II - Hits\n".to_string();

        let parser_output1 = parse_log_line(&log_string1).unwrap();
        let parser_output2 = parse_log_line(&log_string2).unwrap();
        let parser_output3 = parse_log_line(&log_string3).unwrap();

        let expected_output1 = DamageLog::new(
            Utc.with_ymd_and_hms(2024, 7, 2, 20, 31, 28).unwrap(),
            200,
            "Hornet EC-300".to_string(),
            "Hornet EC-300".to_string(),
            "Draclira's Modified Tachyon Beam Laser".to_string(),
            Destination::Dealing,
        );
        let expected_output2 = DamageLog::new(
            Utc.with_ymd_and_hms(2024, 06, 25, 15, 20, 01).unwrap(),
            375,
            "Tek'wka Rokym".to_string(),
            "Paladin".to_string(),
            "Imperial Navy Large EMP Smartbomb".to_string(),
            Destination::Receiving,
        );
        let expected_output3 = DamageLog::new(
            Utc.with_ymd_and_hms(2024, 07, 02, 19, 42, 05).unwrap(),
            153,
            "Kilyavi Alaailaa".to_string(),
            "Capsule".to_string(),
            "Medium Vorton Projector II".to_string(),
            Destination::Dealing,
        );

        assert_eq!(parser_output1, Log::Damage(expected_output1));
        assert_eq!(parser_output2, Log::Damage(expected_output2));
        assert_eq!(parser_output3, Log::Damage(expected_output3));
    }

    #[test]
    fn test_basic_logi_logs() {
        // TODO still need to test logi output
        let log_string1 = "[ 2024.07.02 19:13:23 ] (combat) <color=0xffccff66><b>772</b><color=0x77ffffff><font size=10> remote shield boosted by </font><b><color=0xffffffff><font size=14><color=0xFFFFFFFF> <b>Osprey</b></color></font><color=0xFFB3B3B3> [Drentu]<color=0xFFFFFFFF><b> -</color> </b><color=0x77ffffff><font size=10> - Medium Ancillary Remote Shield Booster</font>\n".to_string();
        let log_string2 = "[ 2024.07.02 20:14:35 ] (combat) <color=0xffccff66><b>665</b><color=0x77ffffff><font size=10> remote shield boosted by </font><b><color=0xffffffff><font size=14><color=0xFF70FF40> <b>Scimitar</b></color></font><color=0xFFFF4040> [Drentu]<color=0xFFFFFFFF><b> -</color> </b><color=0x77ffffff><font size=10> - Large Remote Shield Booster II</font>\n".to_string();

        let parser_output1 = parse_log_line(&log_string1).unwrap();
        let parser_output2 = parse_log_line(&log_string2).unwrap();

        let expected_output1 = LogiLog::new(
            Utc.with_ymd_and_hms(2024, 7, 2, 19, 13, 23).unwrap(),
            772,
            "Drentu".to_string(),
            "Osprey".to_string(),
            "Medium Ancillary Remote Shield Booster".to_string(),
            Destination::Receiving,
        );
        let expected_output2 = LogiLog::new(
            Utc.with_ymd_and_hms(2024, 07, 02, 20, 14, 35).unwrap(),
            665,
            "Drentu".to_string(),
            "Scimitar".to_string(),
            "Large Remote Shield Booster II".to_string(),
            Destination::Receiving,
        );

        assert_eq!(parser_output1, Log::Logi(expected_output1));
        assert_eq!(parser_output2, Log::Logi(expected_output2));
    }

    #[test]
    fn test_header_parsing() {
        let header1 = "------------------------------------------------------------
  Gamelog
  Listener: T'rahk Rokym
  Session Started: 2024.07.02 19:41:40
------------------------------------------------------------\n"
            .to_string();

        let expected_output1 = Some((
            "T'rahk Rokym".to_string(),
            Utc.with_ymd_and_hms(2024, 07, 02, 19, 41, 40).unwrap(),
        ));

        let output1 = parse_log_header(header1);

        assert_eq!(output1, expected_output1)
    }
}
