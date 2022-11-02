use midi_parse::datatypes::DrumTrack;
use midi_parse::map::{process_track_pool, NUMBER_OF_TRACKS, RESOLUTION};
use midi_parse::parse::filter_beat;
use midly::Smf;
use ndarray::{Array, ArrayView, Ix3};
use std::fs;
use structopt::StructOpt;

use iced::{
    canvas::{self, Canvas, Cursor, Frame, Path, Stroke },
    executor, scrollable, Align, Application, Column, Command, Container, Element, Length, Point,
    Rectangle, Scrollable, Settings, Size, Color
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
    let track_pool: Vec<DrumTrack> =
        filter_beat(Smf::parse(&data).expect("could not parse SMF data"), true);
    // get ndarray version
    process_track_pool(&track_pool)
        .expect("Failed to cast tracks into ndarray 4")
        .outer_iter()
        .map(|bar_view: ArrayView<f32, Ix3>| bar_view.to_owned())
        .collect()
}

const CANVAS_WIDTH: u16 = 900;
const CANVAS_HEIGHT: u16 = 240;

#[derive(Debug)]
enum Message {}

struct Bars {
    data: Vec<Array<f32, Ix3>>,
    scroll: scrollable::State,
}

impl Application for Bars {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Bars {
                data: get_midi_data(),
                scroll: scrollable::State::new(),
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
        let bars: Element<_> = self
            .data
            .iter()
            .fold(
                Column::new()
                    .padding(20)
                    .spacing(20)
                    .align_items(Align::Center),
                |column, bar| {
                    column.push(
                        Canvas::new(Bar {
                            bar: bar.to_owned(),
                        })
                        .width(Length::Units(CANVAS_WIDTH))
                        .height(Length::Units(CANVAS_HEIGHT)),
                    )
                },
            )
            .into();

        let scrollable: Element<_> = Scrollable::new(&mut self.scroll)
            .push(Container::new(bars).width(Length::Fill).center_x())
            .into();

        Container::new(scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Debug)]
struct Bar {
    bar: Array<f32, Ix3>,
}

impl canvas::Program<Message> for Bar {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<canvas::Geometry> {
        let bar = &self.bar;
        let mut frame = Frame::new(bounds.size());

        const STEP_WIDTH: f32 = CANVAS_WIDTH as f32 / RESOLUTION as f32;
        const STEP_HEIGHT: f32 = CANVAS_HEIGHT as f32 / NUMBER_OF_TRACKS as f32;
        const STEP_PADDING_Y: f32 = 1.5;
        const EVENT_WIDTH: f32 = STEP_WIDTH / 3.0;


        for track_index in 0..NUMBER_OF_TRACKS + 1 {
            let y: f32 = STEP_HEIGHT * track_index as f32;
            let line_h = Path::line(Point::new(0.0, y), Point::new(CANVAS_WIDTH as f32, y));

            if (track_index) % 2 == 0 {
                frame.stroke(&line_h, 
                    Stroke::default()
                        .with_width(1.2)
                        .with_color(Color::from_rgba(0.2, 0.2, 0.2, 1.0))
                );
            } else {
                frame.stroke(&line_h, 
                    Stroke::default()
                        .with_width(0.5)
                        .with_color(Color::from_rgba(0.2, 0.2, 0.2, 0.8))
                );
            }
        }

        for step_index in 0..RESOLUTION + 1 {
            let x: f32 = STEP_WIDTH * step_index as f32 ;
            let line_v = Path::line(Point::new(x, 0.0), Point::new(x, CANVAS_HEIGHT as f32));
           
            if (step_index) % 8 == 0 {
                frame.stroke(&line_v, 
                    Stroke::default()
                        .with_width(2.0)
                        .with_color(Color::from_rgba(0.0, 0.0, 0.0, 0.9))
                );
            } else if (step_index) % 4 == 0 {
                frame.stroke(&line_v, 
                    Stroke::default()
                        .with_width(2.0)
                        .with_color(Color::from_rgba(0.1, 0.1, 0.1, 0.8))
                );
            } else {
                frame.stroke(&line_v, 
                    Stroke::default()
                        .with_width(1.0)
                        .with_color(Color::from_rgba(0.2, 0.2, 0.2, 0.5))
                );
            }
        }

        bar.outer_iter().enumerate().for_each(|(step_index, arr2)| {    
            arr2.outer_iter().rev().enumerate().for_each(|(track_index, arr1)| {
                match arr1.as_slice() {
                    Some(step) => {
                        let offset = step[1];
                        let velocity = step[0];

                        if velocity > 0.0 {
                            // println!("track_index {} step_index {} velocity {} offset {}", track_index, step_index, velocity, offset);

                            let origin = Point::new(
                                (step_index as f32 + offset) * STEP_WIDTH,
                                track_index as f32 * STEP_HEIGHT + STEP_PADDING_Y
                            );

                            let color = if offset > 0. {
                                    Color::from_rgba(2.0/255.0, 221.0/255.0, 103.0/255.0, (velocity * 0.5) + 0.4)
                                } else {
                                    Color::from_rgba(221.0/255.0, 0./255.0, 0./255.0, (velocity * 0.5) + 0.4)
                                };
    
                            let step = Path::rectangle(origin, Size::new(EVENT_WIDTH, STEP_HEIGHT - 2.0 * STEP_PADDING_Y));
                            frame.fill(&step, color);
                        }
                    }
                    None => println!("!PANIC"),
                }
            });
        }); 

        vec![frame.into_geometry()]
    }
}
