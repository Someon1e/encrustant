use core::num::{NonZeroU16, NonZeroU64};
use core::str::SplitWhitespace;

use crate::search::Ply;

#[derive(Default)]
pub struct SearchTime {
    pondering: Option<bool>,

    mate_in_moves: Option<u8>,

    infinite: bool,

    fixed_time: Option<u64>,

    nodes: Option<u64>,

    depth: Option<u8>,

    white_time: Option<u64>,
    black_time: Option<u64>,
    white_increment: Option<NonZeroU64>,
    black_increment: Option<NonZeroU64>,
    moves_to_go: Option<NonZeroU16>,
}

impl SearchTime {
    #[must_use]
    pub const fn fixed_time(&self) -> Option<u64> {
        self.fixed_time
    }

    #[must_use]
    pub const fn mate_in_moves(&self) -> Option<u8> {
        self.mate_in_moves
    }

    #[must_use]
    pub const fn depth(&self) -> Option<u8> {
        self.depth
    }

    /// Limit of nodes to search
    #[must_use]
    pub const fn nodes(&self) -> Option<u64> {
        self.nodes
    }

    #[must_use]
    pub const fn pondering(&self) -> Option<bool> {
        self.pondering
    }

    #[must_use]
    pub const fn infinite(&self) -> bool {
        self.infinite
    }

    #[must_use]
    pub const fn white_time(&self) -> Option<u64> {
        self.white_time
    }

    #[must_use]
    pub const fn black_time(&self) -> Option<u64> {
        self.black_time
    }

    #[must_use]
    pub const fn white_increment(&self) -> Option<NonZeroU64> {
        self.white_increment
    }

    #[must_use]
    pub const fn black_increment(&self) -> Option<NonZeroU64> {
        self.black_increment
    }
}

pub enum SearchType {
    Normal(SearchTime),
    Perft(u16),
}

pub struct GoParameters {
    search_type: Option<SearchType>,
}

impl GoParameters {
    /// Returns empty parameters
    #[must_use]
    pub const fn empty() -> Self {
        Self { search_type: None }
    }

    pub fn parse(&mut self, args: &mut SplitWhitespace) {
        while let Some(label) = args.next() {
            macro_rules! parse_number {
                () => {
                    args.next().unwrap().parse()
                };
            }
            match label {
                "wtime" | "btime" | "winc" | "binc" | "ponder" | "movetime" | "depth" | "nodes"
                | "infinite" | "movestogo" | "mate" => {
                    match self.search_type {
                        None => {
                            self.search_type = Some(SearchType::Normal(SearchTime::default()));
                        }
                        Some(SearchType::Normal(_)) => {}
                        Some(SearchType::Perft(_)) => {
                            panic!(
                                "search parameters specified when search type already determined"
                            )
                        }
                    }
                    let search_time = match self.search_type {
                        Some(SearchType::Normal(ref mut search_time)) => search_time,
                        _ => unreachable!(),
                    };

                    match label {
                        "wtime" => {
                            assert!(search_time.white_time.is_none(), "Overwritten wtime");
                            search_time.white_time = Some(parse_number!().unwrap());
                        }
                        "btime" => {
                            assert!(search_time.black_time.is_none(), "Overwritten btime");
                            search_time.black_time = Some(parse_number!().unwrap());
                        }

                        "winc" => {
                            assert!(search_time.white_increment.is_none(), "Overwritten winc");
                            if let Ok(winc) = parse_number!() {
                                search_time.white_increment = NonZeroU64::new(winc);
                            } else {
                                search_time.white_increment = None;
                            }
                        }
                        "binc" => {
                            assert!(search_time.black_increment.is_none(), "Overwritten binc");
                            if let Ok(binc) = parse_number!() {
                                search_time.black_increment = NonZeroU64::new(binc);
                            } else {
                                search_time.black_increment = None;
                            }
                        }
                        "ponder" => {
                            assert!(search_time.pondering.is_none(), "Pondering defined twice");
                            search_time.pondering = Some(true);
                        }
                        "movetime" => {
                            assert!(search_time.fixed_time.is_none(), "Overwritten movetime");
                            search_time.fixed_time = Some(parse_number!().unwrap());
                        }
                        "depth" => {
                            assert!(search_time.depth.is_none(), "Overwritten depth");
                            search_time.depth = Some(parse_number!().unwrap());
                        }
                        "nodes" => {
                            assert!(search_time.nodes.is_none(), "Overwritten nodes");
                            search_time.nodes = Some(parse_number!().unwrap());
                        }
                        "infinite" => {
                            assert!(!search_time.infinite, "infinite specified twice");
                            search_time.infinite = true;
                        }
                        "movestogo" => {
                            assert!(search_time.moves_to_go.is_none(), "Overwritten movestogo");
                            search_time.moves_to_go = Some(parse_number!().unwrap());
                        }
                        "mate" => {
                            assert!(search_time.mate_in_moves.is_none(), "Overwritten mate");
                            let mate_in_moves = parse_number!().unwrap();
                            const MAX_MOVES: u32 = (Ply::MAX as u32).div_ceil(2);
                            assert!(mate_in_moves < MAX_MOVES as Ply);
                            search_time.mate_in_moves = Some(mate_in_moves);
                        }
                        _ => unreachable!(),
                    }
                }

                "perft" => {
                    assert!(
                        matches!(self.search_type, None | Some(SearchType::Normal(_))),
                        "perft specified when search type already determined"
                    );
                    self.search_type = Some(SearchType::Perft(
                        parse_number!().expect("Perft depth not specified"),
                    ));
                }

                "searchmoves" => todo!(),

                _ => panic!("Unknown parameter"),
            }
        }
    }

    #[must_use]
    pub const fn search_type(self) -> Option<SearchType> {
        self.search_type
    }
}
