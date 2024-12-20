use core::fmt;
use crossterm::{
    cursor::{Hide, MoveTo},
    execute,
    terminal::{
        Clear,
        ClearType::{All, Purge},
        DisableLineWrap,
    },
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{
    collections::HashMap,
    io::{stdin, stdout},
    time::{Duration, Instant},
};

const SOLVE_TIME: u64 = 300;

fn main() {
    execute!(stdout(), Hide, DisableLineWrap,).unwrap();

    let mut game = GameState::new();
    let start_time = Instant::now();

    'game: while start_time.elapsed() < Duration::from_secs(SOLVE_TIME) && game.accusations_left > 0
    {
        let seconds_remaining = SOLVE_TIME - Instant::now().duration_since(start_time).as_secs();

        'input: loop {
            execute!(stdout(), Clear(All), Clear(Purge), MoveTo(0, 0)).unwrap();

            println!(
                "Time left: {} min {} sec",
                seconds_remaining / 60,
                seconds_remaining % 60
            );
            println!("DEBUG: The culprit is {}", game.culprit);
            if game.current_location == Noun::Place(Locations::Taxi) {
                println!("You are in your TAXI. Where do you want to go?");
                for (index, place) in game.places.iter().enumerate() {
                    let label = format!("({index}) {place}");
                    let mut place_info = String::new();
                    if let Some(&(suspect, item)) = game.visited_places.get(place) {
                        place_info = format!("({}, {})", suspect, item);
                    }
                    println!("{}     {}", label, place_info);
                }
                println!("(Q)uit Game")
            }

            let input = get_input();

            let number = input.parse::<usize>();
            if input == "Q" {
                println!("Thanks for playing!");
                break 'game;
            } else if number.is_ok() && *number.as_ref().unwrap() < game.places.len() {
                game.current_location = game.places[number.unwrap()];
                break 'input;
            } else {
                println!("Invalid input. Please try again.");
                continue 'input;
            }
        }

        let current_index = game
            .places
            .iter()
            .position(|place| *place == game.current_location)
            .unwrap();

        let the_person_here = &game.suspects[current_index];
        let the_item_here = game.items[current_index];
        let debug_is_liar = game.liars.contains(the_person_here);

        println!("You are at the {}", game.current_location);
        println!("{} with the {} is here", the_person_here, the_item_here);
        if debug_is_liar {
            println!("DEBUG: They are a liar.");
        }

        if !game.known_suspects_items.contains(&the_person_here) {
            game.known_suspects_items.push(*the_person_here);
        }
        if !game.known_suspects_items.contains(&the_item_here) {
            game.known_suspects_items.push(the_item_here);
        }
        if !game.visited_places.contains_key(&game.current_location) {
            game.visited_places
                .insert(game.current_location, (*the_person_here, the_item_here));
        }

        if game.accused_suspects.contains(&the_person_here) {
            println!("They are offended that you accused them,");
            println!("and will not help you with your investigation.");
            println!("You go back to your TAXI");
            println!("(press ENTER to continue...)");
            game.current_location = Noun::Place(Locations::Taxi);
            continue 'game;
        }

        println!(
            "(J) J'ACCUSE!     ({} accusations left)",
            game.accusations_left
        );
        println!("(Z) Ask if they know where ZOPHIE THE CAT is.\n(T) Go back to the TAXI.");

        for (index, noun) in game.known_suspects_items.iter().enumerate() {
            println!("({}) Ask about {}", index, noun);
        }

        'input: loop {
            let input = get_input();
            if input == "J" {
                game.accusations_left -= 1;
                if *the_person_here == game.culprit {
                    let time_passed = Instant::now().duration_since(start_time).as_secs();
                    println!("You've cracked the case, Detective!");
                    println!("It was {} who had catnapped ZOPHIE THE CAT.", game.culprit);
                    println!(
                        "Good job! You solved it in {} min, {} sec.",
                        time_passed / 60,
                        time_passed % 60
                    );
                    break 'game;
                } else {
                    game.accused_suspects.push(*the_person_here);
                    println!("You have accused the wrong person, Detective!\nThey will not help you with anymore clues\nYou go back to your TAXI");
                    game.current_location = Noun::Place(Locations::Taxi);
                    break 'input;
                }
            } else if input == "Z" {
                if let Some(clue) = game.zophie_clues.get(&the_person_here) {
                    println!("They give you this clue: Check {}", clue);
                    if !game.known_suspects_items.contains(clue) {
                        if let Noun::Person(_) = clue {
                            game.known_suspects_items.push(*clue);
                        }
                        if let Noun::Thing(_) = clue {
                            game.known_suspects_items.push(*clue);
                        }
                    }
                } else {
                    println!("I don't know anything about ZOPHIE THE CAT.");
                }
                break 'input;
            } else if input == "T" {
                game.current_location = Noun::Place(Locations::Taxi);
                continue 'game;
            } else if input.parse::<usize>().is_ok() {
                let index = input.parse::<usize>().unwrap();
                if index < game.known_suspects_items.len() {
                    let noun_asked_about = game.known_suspects_items[index];
                    println!("DEBUG: You've asked about: {}", noun_asked_about);

                    if noun_asked_about == *the_person_here || noun_asked_about == the_item_here {
                        println!("They give you this clue: 'No comment.'");
                    } else {
                        let clue = game
                            .clues
                            .get(&Clue(*the_person_here, noun_asked_about))
                            .unwrap();
                        println!("They give you this clue: {}", clue);
                        if !game.known_suspects_items.contains(&clue) {
                            if let Noun::Person(_) = clue {
                                game.known_suspects_items.push(*clue);
                            }
                            if let Noun::Thing(_) = clue {
                                game.known_suspects_items.push(*clue);
                            }
                        }
                    }
                    break 'input;
                } else {
                    println!("Invalid selection. Please try again.");
                    continue 'input;
                }
            } else {
                println!("Invalid selection. Please try again.");
                continue 'input;
            }
        }
        println!("Press ENTER to return to the TAXI...");
        game.current_location = Noun::Place(Locations::Taxi);
        let _ = get_input();

        //Game loss logic -- TODO game win logic
        if start_time.elapsed() >= Duration::from_secs(SOLVE_TIME) {
            println!("You have run out of time!");
        } else if game.accusations_left == 0 {
            //TODO - what if last accusation is culprit?
            println!("You have accused too many innocent people!");
        }
        let culprit_index = game
            .suspects
            .iter()
            .enumerate()
            .filter(|(_index, suspect)| **suspect == game.culprit)
            .collect::<Vec<_>>()[0]
            .0;
        println!(
            "It was {} at the {} with the {} who catnapped her!",
            game.culprit, game.places[culprit_index], game.items[culprit_index]
        )
    }
}

struct GameState {
    known_suspects_items: Vec<Noun>,
    visited_places: HashMap<Noun, (Noun, Noun)>,
    current_location: Noun,
    accused_suspects: Vec<Noun>,
    accusations_left: u8,
    liars: Vec<Noun>,
    culprit: Noun,
    suspects: Vec<Noun>,
    items: Vec<Noun>,
    places: Vec<Noun>,
    ///Noun is the response given (always item or suspect)
    clues: HashMap<Clue, Noun>,
    zophie_clues: HashMap<Noun, Noun>,
}

impl GameState {
    fn new() -> GameState {
        let mut game = GameState {
            known_suspects_items: vec![],
            visited_places: HashMap::new(),
            current_location: Noun::Place(Locations::Taxi),
            accused_suspects: vec![],
            accusations_left: 3,
            liars: Suspects::random_sample(3),
            culprit: Suspects::random_sample(1)[0],
            suspects: Suspects::get_shuffled_list(),
            items: Items::get_shuffled_list(),
            places: Locations::get_shuffled_list(),
            clues: HashMap::new(),
            zophie_clues: HashMap::new(),
        };
        let mut rng = thread_rng();
        let items_and_suspects = vec![
            game.items
                .clone()
                .into_iter()
                .enumerate()
                .collect::<Vec<(usize, Noun)>>(),
            game.suspects
                .clone()
                .into_iter()
                .enumerate()
                .collect::<Vec<(usize, Noun)>>(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<(usize, Noun)>>();

        for interviewee in &game.suspects {
            let is_liar = game.liars.contains(&interviewee);

            for (index, subject) in items_and_suspects.iter() {
                let mut response: Option<Noun> = None;
                let choice = rng.gen_bool(0.5);

                if let Noun::Thing(_) = subject {
                    if choice {
                        response = Self::new_clue(is_liar, &game.places, *index);
                    } else {
                        response = Self::new_clue(is_liar, &game.suspects, *index);
                    }
                }
                if let Noun::Person(_) = subject {
                    if choice {
                        response = Self::new_clue(is_liar, &game.places, *index);
                    } else {
                        response = Self::new_clue(is_liar, &game.items, *index);
                    }
                }
                if let Some(noun) = response {
                    game.clues.insert(Clue(*interviewee, *subject), noun);
                }
            }
        }

        let culprit_index = game
            .suspects
            .iter()
            .position(|noun| *noun == game.culprit)
            .unwrap();

        for interviewee in Suspects::random_sample(3) {
            let kind_of_clue = rng.gen_range(0..=2);
            let clue: Noun;

            if kind_of_clue == 0 {
                clue = Self::new_zophie_clue(&game, &game.suspects, culprit_index, interviewee);
            } else if kind_of_clue == 1 {
                clue = Self::new_zophie_clue(&game, &game.places, culprit_index, interviewee);
            } else {
                clue = Self::new_zophie_clue(&game, &game.items, culprit_index, interviewee);
            }
            game.zophie_clues.insert(interviewee, clue);
        }
        game
    }

    fn new_clue(is_liar: bool, nouns: &Vec<Noun>, culprit_index: usize) -> Option<Noun> {
        if is_liar {
            Some(Self::lie_about_noun(nouns, culprit_index))
        } else {
            Some(nouns[culprit_index].clone())
        }
    }

    fn new_zophie_clue(
        game: &GameState,
        nouns: &Vec<Noun>,
        true_index: usize,
        interviewee: Noun,
    ) -> Noun {
        if game.liars.contains(&interviewee) {
            Self::lie_about_noun(nouns, true_index)
        } else {
            nouns[true_index].clone()
        }
    }

    fn lie_about_noun(nouns: &Vec<Noun>, culprit_index: usize) -> Noun {
        let index_of_lie = thread_rng().gen_range(0..8);

        let lie = nouns
            .iter()
            .enumerate()
            .filter(|(index, _noun)| *index != culprit_index)
            .collect::<Vec<_>>()[index_of_lie]
            .1
            .clone();
        lie
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Suspects {
    DukeHautdog,
    MaximumPowers,
    BillMonopolis,
    SenatorSchmear,
    MrsFeathertoss,
    DrJeanSplicer,
    RafflesTheClown,
    EspressaToffeepot,
    CecilEdgarVanderton,
}
impl fmt::Display for Suspects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Suspects::DukeHautdog => write!(f, "DUKE HAUTDOG"),
            Suspects::MaximumPowers => write!(f, "MAXIMUM POWERS"),
            Suspects::BillMonopolis => write!(f, "BILL MONOPOLIS"),
            Suspects::SenatorSchmear => write!(f, "SENATOR SCHMEAR"),
            Suspects::MrsFeathertoss => write!(f, "MRS. FEATHERTOSS"),
            Suspects::DrJeanSplicer => write!(f, "DR. JEAN SPLICER"),
            Suspects::RafflesTheClown => write!(f, "RAFFLES THE CLOWN"),
            Suspects::EspressaToffeepot => write!(f, "ESPRESSA TOFFEEPOT"),
            Suspects::CecilEdgarVanderton => write!(f, "CECIL EDGAR VANDERTON"),
        }
    }
}
impl Suspects {
    fn random_sample(n: usize) -> Vec<Noun> {
        let mut source = vec![
            Noun::Person(Suspects::DukeHautdog),
            Noun::Person(Suspects::MaximumPowers),
            Noun::Person(Suspects::BillMonopolis),
            Noun::Person(Suspects::SenatorSchmear),
            Noun::Person(Suspects::MrsFeathertoss),
            Noun::Person(Suspects::DrJeanSplicer),
            Noun::Person(Suspects::RafflesTheClown),
            Noun::Person(Suspects::EspressaToffeepot),
            Noun::Person(Suspects::CecilEdgarVanderton),
        ];

        let mut suspects = vec![];

        for _ in 0..n {
            let suspect = source.remove(thread_rng().gen_range(0..source.len()));
            suspects.push(suspect);
        }
        suspects
    }
    fn get_shuffled_list() -> Vec<Noun> {
        let mut source = vec![
            Noun::Person(Suspects::DukeHautdog),
            Noun::Person(Suspects::MaximumPowers),
            Noun::Person(Suspects::BillMonopolis),
            Noun::Person(Suspects::SenatorSchmear),
            Noun::Person(Suspects::MrsFeathertoss),
            Noun::Person(Suspects::DrJeanSplicer),
            Noun::Person(Suspects::RafflesTheClown),
            Noun::Person(Suspects::EspressaToffeepot),
            Noun::Person(Suspects::CecilEdgarVanderton),
        ];
        source.shuffle(&mut thread_rng());

        source
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Items {
    Flashlight,
    Candlestick,
    RainbowFlag,
    HamsterWheel,
    AnimeVhsTape,
    JarOfPickles,
    OneCowboyBoot,
    CleanUnerpants,
    FiveDollarGiftCard,
}
impl fmt::Display for Items {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Items::Flashlight => write!(f, "FLASHLIGHT"),
            Items::Candlestick => write!(f, "CANDLESTICK"),
            Items::RainbowFlag => write!(f, "RAINBOW FLAG"),
            Items::HamsterWheel => write!(f, "HAMSTER WHEEL"),
            Items::AnimeVhsTape => write!(f, "ANIME VHS TAPE"),
            Items::JarOfPickles => write!(f, "JAR OF PICKLES"),
            Items::OneCowboyBoot => write!(f, "ONE COWBOY BOOT"),
            Items::CleanUnerpants => write!(f, "CLEAN UNDERPANTS"),
            Items::FiveDollarGiftCard => write!(f, "FIVE DOLLAR GIFT CARD"),
        }
    }
}
impl Items {
    fn get_shuffled_list() -> Vec<Noun> {
        let mut source = vec![
            Noun::Thing(Items::Flashlight),
            Noun::Thing(Items::Candlestick),
            Noun::Thing(Items::RainbowFlag),
            Noun::Thing(Items::HamsterWheel),
            Noun::Thing(Items::AnimeVhsTape),
            Noun::Thing(Items::JarOfPickles),
            Noun::Thing(Items::OneCowboyBoot),
            Noun::Thing(Items::CleanUnerpants),
            Noun::Thing(Items::FiveDollarGiftCard),
        ];
        source.shuffle(&mut thread_rng());

        source
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Locations {
    Zoo,
    OldBarn,
    DuckPond,
    CityHall,
    HipsterCafe,
    BowlingAlley,
    VideoGameMuseum,
    UniversityLibrary,
    AlbinoAlligatorPit,
    Taxi,
}
impl fmt::Display for Locations {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locations::Zoo => write!(f, "ZOO"),
            Locations::OldBarn => write!(f, "OLD BARN"),
            Locations::DuckPond => write!(f, "DUCK POND"),
            Locations::CityHall => write!(f, "CITY HALL"),
            Locations::HipsterCafe => write!(f, "HIPSTER CAFE"),
            Locations::BowlingAlley => write!(f, "BOWLING ALLEY"),
            Locations::VideoGameMuseum => write!(f, "VIDEO GAME MUSEUM"),
            Locations::UniversityLibrary => write!(f, "UNIVERSITY LIBRARY"),
            Locations::AlbinoAlligatorPit => write!(f, "ALBINO ALLIGATOR PIT"),
            Locations::Taxi => write!(f, "TAXI"),
        }
    }
}
impl Locations {
    fn get_shuffled_list() -> Vec<Noun> {
        let mut source = vec![
            Noun::Place(Locations::Zoo),
            Noun::Place(Locations::OldBarn),
            Noun::Place(Locations::DuckPond),
            Noun::Place(Locations::CityHall),
            Noun::Place(Locations::HipsterCafe),
            Noun::Place(Locations::BowlingAlley),
            Noun::Place(Locations::VideoGameMuseum),
            Noun::Place(Locations::UniversityLibrary),
            Noun::Place(Locations::AlbinoAlligatorPit),
        ];
        source.shuffle(&mut thread_rng());

        source
    }
}
///0th noun is the interviewee, 1st is the noun asked about (always suspect or item)
#[derive(Debug, PartialEq, Eq, Hash)]
struct Clue(Noun, Noun);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Noun {
    Person(Suspects),
    Place(Locations),
    Thing(Items),
}

impl fmt::Display for Noun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Noun::Person(suspect) => write!(f, "{}", suspect),
            Noun::Place(location) => write!(f, "{}", location),
            Noun::Thing(item) => write!(f, "{}", item),
        }
    }
}

fn get_input() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().to_uppercase()
}
