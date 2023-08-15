use sauron::*;

pub struct Spinner {
    size: usize,
}

impl Spinner {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl<MSG> Component<MSG, ()> for Spinner
where
    MSG: 'static,
{
    fn view(&self) -> Node<MSG> {
        svg(
            [
                Self::class_ns("spinner"),
                view_box([0, 0, self.size, self.size]),
                style! {
                    width: px(self.size),
                    height: px(self.size),
                },
            ],
            [circle(
                [
                    Self::class_ns("path"),
                    cx(self.size / 2),
                    cy(self.size / 2),
                    r(self.size / 3),
                    fill("none"),
                    stroke_width(self.size / 10),
                ],
                [],
            )],
        )
    }

    fn update(&mut self, _msg: MSG) -> Effects<MSG, ()> {
        Effects::none()
    }

    fn stylesheet() -> Vec<String> {
        vec![
            jss! {
            ".spinner": {
                animation: "rotate 1s linear infinite",
                z_index: 2,
                position: "relative",
            },

             ".path": {
                stroke: "black",
                stroke_linecap: "round",
                animation: "dash .7s ease-in-out infinite",
              },
            },
            jss_with_media! {
                "@keyframes rotate": {
                  "100%": {
                    transform: "rotate(360deg)",
                  }
                },
            },
            jss_with_media! {
                "@keyframes dash": {
                  "0%": {
                    stroke_dasharray: "1, 150",
                    stroke_dashoffset: 0,
                  },
                  "50%": {
                    stroke_dasharray: "90, 150",
                    stroke_dashoffset: -35,
                  },
                  "100%": {
                    stroke_dasharray: "90, 150",
                    stroke_dashoffset: -124,
                  },
                },

            },
        ]
    }
}
