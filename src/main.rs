use lastfm::Client;
use tokio::time::sleep;
use std::env;

use std::fs::File;
use std::{time::Duration, io::{self, BufRead, BufReader}};

use futures_util::pin_mut;
use futures_util::stream::StreamExt;


//account details  TODO turn into convars or external config


//width height
static HISTORY_LENGTH: i32 = 10;
static MAX_LENGTH: usize = 55;
// height is calculated by hand for now
// >> if not set correctly it might not redraw the entire block
// conversions are: 1x length + 8 misc
static ERASE_LENGTH: i32 = 18;

//----------------word art stuff starts here------------------------
//draw delay in milliseconds
static DRAW_DELAY: u64 = 10;
//update time in seconds
static UPDATE_PERIOD: u64 = 10;

static TITLE_PATH: &str = "art.txt";
static TITLE_WIDTH: usize = 56;
//window width can't go lower than title width, duh
static WINDOW_WIDTH: usize = 70;
//character set
static VERT_LINE1: char = '║';
static HOR_LINE1: char = '-';
static HOR_LINE2: char = '═';

//corners go from top left clockwise
static CORNER1: char = '╔';
static CORNER2: char = '╗';
static CORNER3: char = '╝';
static CORNER4: char = '╚';


fn read_title() -> io::Result<()> {
    let file = File::open(TITLE_PATH)?;
    let reader = BufReader::new(file);
    let mut offset = String::new();
    let center = WINDOW_WIDTH/2 - TITLE_WIDTH/2;
    for _i in 0.. center{
        offset.push(' ');

    }

    for line in reader.lines() {
        let line = line?;
        println!("{}{}",offset,line);
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}

fn process_row(mut string: String) -> String {
    if string.chars().count() > MAX_LENGTH {
        let (first, _last) = string.split_at_mut(MAX_LENGTH);
        let string2: String=first.to_string();
        return format!("{}{} {}",VERT_LINE1,string2,VERT_LINE1);
    }
    else {
        for _i in string.chars().count()..MAX_LENGTH {
            string.push(' ');
        }
        return format!("{}{} {}",VERT_LINE1,string,VERT_LINE1);
    }

}

fn offset_output(string: String) -> String {
    let mut offset = String::new();
    let center = WINDOW_WIDTH/2 - (MAX_LENGTH+1)/2;
    for _i in 0.. center{
        offset.push(' ');
    }
    return format!("{}{}",offset,string);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("\x1B[2J\x1B[H");
    print!("\n");


    let mut api_key = "";
    let mut username = "";

    match env::var("LASTRS_KEY") {
        Ok(val) => {
           api_key = val.clone().leak();
        }
        Err(_e) => println!("Couldn't read LASTRS_KEY"),
    }
    match env::var("LASTRS_USR") {
        Ok(val) => {
           username = val.clone().leak();
        }
        Err(_e) => println!("Couldn't read LASTRS_USR"),
    }



   
    read_title().ok();

    print!("\n");
    let client = Client::builder()
    .api_key(&api_key)
    .username(&username)
    .build();
    let mut first: bool = true;
    loop {
        if !first
        {
            for _n in 0..ERASE_LENGTH {
                sleep(Duration::from_millis(50)).await;
                print!("\x1b[1A");
                print!("\x1b[2K");
            }
        }
        first = false;

        let mut header1: String = String::new();
        for _n in 0..MAX_LENGTH+1{
            header1.push(HOR_LINE1);
        }

        //now playing top half
        println!("{}",offset_output(format!("{}{}{}",CORNER1,header1,CORNER2)));

        sleep(Duration::from_millis(DRAW_DELAY)).await;

        let string: String;
        match client.now_playing().await {

            Ok(Some(track)) => {
                string = format!("> Now playing: {} - {}", track.artist.name, track.name);
            }
            Ok(None) => {
                string = "X No track currently playing.".to_string();
            }
            Err(e) => {
                string = format!("  ⚠ Error fetching now playing track: {}", e);
            }

        }
        println!("{}", offset_output(process_row(string)));
        println!("{}", offset_output(format!("{}{}{}",CORNER4,header1,CORNER3)));
        sleep(Duration::from_millis(DRAW_DELAY)).await;


        println!("");
        sleep(Duration::from_millis(DRAW_DELAY)).await;

        let mut header3: String = String::new();
        for _n in 0..MAX_LENGTH+1{
            header3.push(HOR_LINE2);
        }

        let mut header2: String = "< HISTORY >".to_string();
        for _n in 11..MAX_LENGTH+1{
            header2.push(HOR_LINE2);
        }
        //recents top
        println!("{}",offset_output(format!("{}{}{}",CORNER1,header2,CORNER2)));
        sleep(Duration::from_millis(DRAW_DELAY)).await;

        let tracks = client.clone().all_tracks().await?;
        let mut index = 0;
        let recent_tracks = tracks.into_stream();
        pin_mut!(recent_tracks); // needed for iteration
        while let Some(track) = recent_tracks.next().await{
            match track {
                Ok(track) => {
                    let string = format!("{} - {}",track.artist.name,track.name);
                    println!("{}",offset_output(process_row(string)));
                    if index >HISTORY_LENGTH{break;}
                    index += 1;
                }

                Err(e) => {
                    println!("Error fetching data: {:?}", e);
                }
            }
        }

        sleep(Duration::from_millis(DRAW_DELAY)).await;
        println!("{}",offset_output(format!("{}{}{}",CORNER4,header3,CORNER3)));
        sleep(Duration::from_secs(UPDATE_PERIOD)).await; // Wait 10 seconds before checking again
    }
}
