// #[macro_use]
use anyhow::{anyhow, Error, Result};
use core::str::FromStr;
use lazy_static::lazy_static;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{none_of, one_of},
    combinator::opt,
    multi::{many0, many1},
    sequence::tuple,
    IResult,
};
use std::{
    ascii::AsciiExt,
    collections::{HashMap, HashSet},
};

#[derive(Debug, PartialEq, Clone, Hash)]
pub enum CalqAst {
    Sum(Box<CalqAst>, Box<CalqAst>),
    Subtract(Box<CalqAst>, Box<CalqAst>),
    Atom(usize, Essence),
}

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
pub struct Essence {
    yin_yang: isize,
    water_fire: isize,
    earth_air: isize,
}

lazy_static! {
    static ref ESSENCE_STRING_MAP: HashMap<Essence,&'static str> = {
        let mut m : HashMap<Essence,&'static str> =  HashMap::new();
        // Base Kinds
        m.insert( Essence::UNALIGNED,"unaligned");
        m.insert( Essence::YIN,"yin");
        m.insert( Essence::YANG,"yang");
        m.insert( Essence::WATER,"water");
        m.insert( Essence::FIRE,"fire");
        m.insert( Essence::EARTH,"earth");
        m.insert( Essence::AIR,"air");
        // Double Kinds
        m.insert( Essence::SUN_LIGHT,"sun/light");
        m.insert( Essence::SKY,"sky");
        m.insert( Essence::PURITY_HEALING,"purity/healing");
        m.insert( Essence::GRAVITY,"gravity");
        m.insert( Essence::HEAT,"heat");
        m.insert( Essence::ICE,"ice");
        m.insert( Essence::MUD,"mud");
        m.insert( Essence::MAGMA,"magma");
        m.insert( Essence::PLASMA,"plasma");
        m.insert( Essence::ABYSS_DARKNESS_MOON,"abyss/darkness/moon");
        m.insert( Essence::VACUUM_VOID,"vacuum/void");
        m.insert( Essence::STONE,"stone");
        // Triple Kinds
        m.insert( Essence::LIGHTNING        ,"lightning");
        m.insert( Essence::CLOUD_MIST				,"cloud/mist");
        m.insert( Essence::WOOD_LIFE				,"wood/life");
        m.insert( Essence::METAL						,"metal");
        m.insert( Essence::DESTRUCTION_DEATH,"destruction/death");
        m.insert( Essence::ACID_CORROSION	  ,"acid/corrosion");
        m.insert( Essence::MIASMA						,"miasma");
        m.insert( Essence::FORCE						,"force");
        m
    };
    static ref ESSENCE_SET :HashSet<Essence>= ESSENCE_STRING_MAP.keys().cloned().collect();
    pub static ref ESSENCE_ORDER :[Essence;27]= [
            // Triple Kinds
            Essence::LIGHTNING,
            Essence::CLOUD_MIST,
            Essence::WOOD_LIFE,
            Essence::METAL,
            Essence::DESTRUCTION_DEATH,
            Essence::ACID_CORROSION,
            Essence::MIASMA,
            Essence::FORCE,
            // Double Kinds
            Essence::SUN_LIGHT,
            Essence::SKY,
            Essence::PURITY_HEALING,
            Essence::GRAVITY,
            Essence::HEAT,
            Essence::ICE,
            Essence::MUD,
            Essence::MAGMA,
            Essence::PLASMA,
            Essence::ABYSS_DARKNESS_MOON,
            Essence::VACUUM_VOID,
            Essence::STONE,
            // Base kinds
            Essence::YIN,
            Essence::YANG,
            Essence::WATER,
            Essence::FIRE,
            Essence::EARTH,
            Essence::AIR,
            // unaligned
            Essence::UNALIGNED,
        ];
    static ref STRING_ESSENCE_MAP: HashMap<&'static str,Essence> = {
       let mut m :  HashMap<&'static str,Essence> = HashMap::new();
       for e in ESSENCE_SET.iter() {
let s = ESSENCE_STRING_MAP.get(e).unwrap();
           m.insert(s,*e);
for sub_s in s.split("/") {
           m.insert(sub_s,*e);

}
       }
       m
    };
}

impl Essence {
    pub const fn new(yin_yang: isize, water_fire: isize, earth_air: isize) -> Self {
        Essence {
            yin_yang,
            water_fire,
            earth_air,
        }
    }

    /// Combines |other| and |self| by adding like elements
    pub const fn combine(&self, other: &Self) -> Self {
        Essence {
            yin_yang: self.yin_yang + other.yin_yang,
            water_fire: self.water_fire + other.water_fire,
            earth_air: self.earth_air + other.earth_air,
        }
    }

    /// Removes |other| from |self| by subtracting like elements
    pub const fn remove(&self, other: &Self) -> Self {
        Essence {
            yin_yang: self.yin_yang - other.yin_yang,
            water_fire: self.water_fire - other.water_fire,
            earth_air: self.earth_air - other.earth_air,
        }
    }

    /// Check if (ignoring all ratios) |self| and |other| are the same kind of essence
    pub const fn contains(&self, other: &Self) -> bool {
        // true if a && b >0 <0 or equal, false otherwise
        const fn same_direction(a: isize, b: isize) -> bool {
            return (a > 0 && b > 0) || (a < 0 && b < 0) || (a == 0 && b == 0);
        }
        return same_direction(self.yin_yang, other.yin_yang)
            && same_direction(self.water_fire, other.water_fire)
            && same_direction(self.earth_air, other.earth_air);
    }

    /// Turns |self| into a string ignoring ratios
    fn to_string_simple(&self) -> String {
        for e in ESSENCE_ORDER.iter() {
            if self.contains(&e) {
                return ESSENCE_STRING_MAP.get(&e).unwrap_or(&"unknown").to_string();
            }
        }
        return "unknown".to_string();
    }

    /// takes strings like "lightning" and returns them as Essence
    pub(crate) fn from_string_simple(x: String) -> Option<Self> {
        STRING_ESSENCE_MAP
            .get(x.to_ascii_lowercase().trim())
            .copied()
    }

    /// Turns Essence into a string like "2*sun/light+1*lightning"
    pub fn to_string(&self) -> String {
        // mutable copy of self
        let mut cpy = self.clone();
        // map of essence to how much of it there is
        let mut set: HashMap<Essence, usize> = HashMap::new();
        // gather components into set
        while cpy != Self::UNALIGNED {
            // while not empty look for matching essence and remove it,
            // placing it in the set
            for e in ESSENCE_ORDER.iter() {
                if cpy.contains(e) {
                    cpy = cpy.remove(e);
                    set.insert(*e, set.get(e).unwrap_or(&0) + 1);
                };
            }
        }
        // turn set into string
        let mut s = String::new();
        for e in ESSENCE_ORDER.iter() {
            if let Some(num) = set.get(e) {
                if s == String::new() || *e != Self::UNALIGNED {
                    // if there is already essence in the output, add a + operation
                    if s != String::new() {
                        s.push_str("+");
                    }
                    // only add unaligned to output if output is empty
                    // only show the number multiple iff it is greater than 1
                    if *num == 1 {
                        s.push_str(&e.to_string_simple());
                    } else {
                        s.push_str(&num.to_string());
                        s.push_str("*");
                        s.push_str(&e.to_string_simple());
                    }
                }
            }
        }
        return s;
    }
    /// multiply the amount of essence by  |i|
    pub fn product(&self, i: usize) -> Self {
        let i: isize = i.try_into().unwrap();
        Essence {
            yin_yang: self.yin_yang * i,
            water_fire: self.water_fire * i,
            earth_air: self.earth_air * i,
        }
    }

    // Base Kinds
    const UNALIGNED: Essence = Self::new(0, 0, 0);
    const YIN: Essence = Self::new(-1, 0, 0);
    const YANG: Essence = Self::new(1, 0, 0);
    const WATER: Essence = Self::new(0, -1, 0);
    const FIRE: Essence = Self::new(0, 1, 0);
    const EARTH: Essence = Self::new(0, 0, -1);
    const AIR: Essence = Self::new(0, 0, 1);

    // Double Kinds
    const SUN_LIGHT: Self = Self::new(1, 1, 0);
    const SKY: Self = Self::new(1, 0, 1);
    const PURITY_HEALING: Self = Self::new(1, -1, 0);
    const GRAVITY: Self = Self::new(1, -1, 0);
    const HEAT: Self = Self::new(0, 1, 1);
    const ICE: Self = Self::new(0, -1, 1);
    const MUD: Self = Self::new(0, -1, -1);
    const MAGMA: Self = Self::new(0, 1, -1);
    const PLASMA: Self = Self::new(-1, 1, 0);
    const ABYSS_DARKNESS_MOON: Self = Self::new(-1, -1, 0);
    const VACUUM_VOID: Self = Self::new(-1, 0, 1);
    const STONE: Self = Self::new(-1, 0, -1);

    // Triple Kinds
    pub const LIGHTNING: Self = Self::new(1, 1, 1);
    pub const CLOUD_MIST: Self = Self::new(1, -1, 1);
    pub const WOOD_LIFE: Self = Self::new(1, -1, -1);
    pub const METAL: Self = Self::new(1, 1, -1);
    pub const DESTRUCTION_DEATH: Self = Self::new(-1, 1, 1);
    pub const ACID_CORROSION: Self = Self::new(-1, -1, -1);
    pub const MIASMA: Self = Self::new(-1, -1, 1);
    pub const FORCE: Self = Self::new(-1, 1, -1);
}

impl Eq for Essence {}

impl std::fmt::Display for Essence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_string())
    }
}
fn parse_whitespace(s: &str) -> IResult<&str, ()> {
    let (s, _) = many0(nom::character::complete::one_of(" \t\n\r"))(s)?;
    Ok((s, ()))
}

// fn parse_sum_essence(s: &str) -> IResult<&str, CalqAst> {
//     let (s, _) = parse_whitespace(s)?;
//     let (s, a) = parse_essence(s)?;
//     let (s, _) = parse_whitespace(s)?;
//     let (s, _) = tag("+")(s)?;
//     let (s, _) = parse_whitespace(s)?;
//     let (s, b) = parse_essence(s)?;
//     let (s, _) = parse_whitespace(s)?;
//     Ok((s, CalqAst::Sum(Box::new(a), Box::new(b))))
// }
//
// // type PError = nom::Err<dyn ParseError<String>>;
// fn parse_subtract_essence(s: &str) -> IResult<&str, CalqAst> {
//     let (s, _) = parse_whitespace(s)?;
//     let (s, a) = parse_essence(s)?;
//     let (s, _) = parse_whitespace(s)?;
//     let (s, _) = tag("-")(s)?;
//     let (s, _) = parse_whitespace(s)?;
//     let (s, b) = parse_essence(s)?;
//     let (s, _) = parse_whitespace(s)?;
//     Ok((s, CalqAst::Subtract(Box::new(a), Box::new(b))))
// }

impl CalqAst {
    pub fn eval(&self) -> Essence {
        match self {
            Self::Sum(a, b) => {
                let a = a.eval();
                let b = b.eval();
                a.combine(&b)
            }
            Self::Subtract(a, b) => {
                let a = a.eval();
                let b = b.eval();
                a.remove(&b)
            }
            Self::Atom(i, e) => e.product(*i),
        }
    }
}

/// parse the simpleist essence calculation (x*e) or (e)
/// where x is an integer and e is essence
fn parse_atom_Calq(s: &str) -> IResult<&str, CalqAst> {
    fn i_product(s: &str) -> IResult<&str, usize> {
        let (s, i_str) = many1(one_of("0123456789"))(s)?;
        let (s, _) = parse_whitespace(s)?;
        let (s, _) = tag("*")(s)?;
        let (s, _) = parse_whitespace(s)?;
        let i_str: String = i_str.iter().collect();
        if let Ok(i) = usize::from_str(&i_str) {
            Ok((s, i))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                s,
                nom::error::ErrorKind::Fix,
            )))
        }
    }
    let (s, _) = parse_whitespace(s)?;
    let (s, i_opt) = nom::combinator::opt(i_product)(s)?;
    let (s, _) = parse_whitespace(s)?;
    let (s, word) = many1(none_of(" \t\r\n()+-*0123456789"))(s)?;
    let (s, _) = parse_whitespace(s)?;
    let word: String = word.iter().collect::<String>().to_ascii_lowercase();
    let i: usize = i_opt.unwrap_or(1);
    if let Some(essence) = STRING_ESSENCE_MAP.get(&word.as_str()) {
        Ok((s, CalqAst::Atom(i, *essence)))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            s,
            nom::error::ErrorKind::Fix,
        )))
        // return Err(anyhow!("``{}`` is not a known form of essence", word));
    }
}

/// Parse an essence calulation with '()' around it
fn parse_Calq_with_parens(s: &str) -> IResult<&str, CalqAst> {
    let (s, _) = parse_whitespace(s)?;
    let (s, _) = tag("(")(s)?;
    let (s, _) = parse_whitespace(s)?;
    let (s, ret) = parse_Calq(s)?;
    let (s, _) = parse_whitespace(s)?;
    let (s, _) = tag(")")(s)?;
    let (s, _) = parse_whitespace(s)?;
    return Ok((s, ret));
}

/// Parse a full essence calqulation (with +, -, and unary -)
fn parse_Calq(s: &str) -> IResult<&str, CalqAst> {
    let p = || alt((parse_atom_Calq, parse_Calq_with_parens));
    let (s, _) = parse_whitespace(s)?;
    let (s, opt_unary_subtraction) = opt(tag("-"))(s)?;
    let (s, _) = parse_whitespace(s)?;
    let (s, ast1) = p()(s)?;
    let (s, _) = parse_whitespace(s)?;
    let (s, exprs) = many0(tuple((alt((tag("+"), tag("-"))), p())))(s)?;
    let (s, _) = parse_whitespace(s)?;
    let mut ret = if opt_unary_subtraction.is_some() {
        // -x == 0-x
        CalqAst::Subtract(
            Box::new(CalqAst::Atom(1, Essence::UNALIGNED)),
            Box::new(ast1),
        )
    } else {
        ast1
    };

    for (op, expr) in exprs {
        if op == "+" {
            ret = CalqAst::Sum(Box::new(ret), Box::new(expr));
        } else {
            // op == '-'
            ret = CalqAst::Subtract(Box::new(ret), Box::new(expr));
        }
    }
    return Ok((s, ret));
}

impl std::str::FromStr for CalqAst {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // let s2 = s.to_string();
        if let Ok((_, x)) = parse_Calq(s) {
            Ok(x)
        } else {
            Err(anyhow!("Parseing Failed"))
        }
    }
}
