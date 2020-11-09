use structopt::StructOpt;
use std::fs;
use midly::Smf;
use midi_parse::parse::filter_beat;
use midi_parse::datatypes::DrumTrack;
use midi_parse::map::{process_track_pool, RESOLUTION, NUMBER_OF_TRACKS};
use ndarray::{Array, Ix3, ArrayView};

use iced::{
    canvas::{self, Canvas, Frame, Path, Stroke, Cursor},
    Element, Length, Column, Point, Align,
    executor, Application, Command, Settings, Rectangle, Container, Text
};


 /* Run this program to display the parsed content of ONE file, no more */

// parse args in a clean struct
#[derive(Debug, StructOpt)]
#[structopt(name = "parser-cli", about = "MIDI beat Dataset Builder")]
struct Opt {
    /// Input path
    #[structopt(short, long)]
    input: String,
}

fn main() -> iced::Result {
    /* DISPLAY */
    Bars::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

fn get_midi_data() -> Vec<Array<f32, Ix3>> {
    let opt = Opt::from_args();
    let mut file_input_error_message = "failed to read input ".to_owned();
    file_input_error_message.push_str(&opt.input);

    // read SMF file
    let data = fs::read(&opt.input).expect(&file_input_error_message);
    // parse midi data
    let track_pool: Vec<DrumTrack> = filter_beat(Smf::parse(&data)
        .expect("could not parse SMF data"));
    // get ndarray version
    process_track_pool(&track_pool)
        .expect("Failed to cast tracks into ndarray 4")
        .outer_iter()
        .map(|bar_view: ArrayView<f32, Ix3>| bar_view.to_owned())
        .collect()
}

#[derive(Debug)]
enum Message {
    AddBar
}

struct Bars {
    data: Vec<Array<f32, Ix3>>,
}

impl Application for Bars {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Bars {
                data: get_midi_data(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Beat Grid display")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let bars = self.data
            .iter()
            .fold(
            Column::new().spacing(10),
            |column, bar| {
                let bar = Bar {bar: bar.clone()};
                column.push(bar.view())
            }
        );

        let content = Column::new()
            .align_items(Align::Center)
            .spacing(20)
            .push(bars);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Debug)]
struct Bar {
    bar: Array<f32, Ix3>
}

impl Bar {
    pub fn view<'a>(&'a mut self) -> Element<'a, Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<'a> canvas::Program<Message> for Bar {
    fn draw(
        &self,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {

        let mut frame = Frame::new(bounds.size());

        let line = Path::line(Point::ORIGIN, Point::new(40.0, 0.0));
        frame.stroke(&line, Stroke::default().with_width(2.0));

        vec![frame.into_geometry()]
    }
}
