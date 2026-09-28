//! Graha relationships: natural, temporary and compound.
//!
//! Specification: `docs/phase2/DESIGN.md` section 4.3.
//! Rahu and Ketu have no relationships in Phase 2 (variant V-2): every
//! function here returns `None` for them.

use lagn_ephem::Graha;
use serde::{Deserialize, Serialize};

use crate::rasi::Rasi;

/// Naisargika (natural, permanent) relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NaturalRelation {
    Friend,
    Neutral,
    Enemy,
}

/// Tatkalika (temporary) relationship, from chart positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporaryRelation {
    Friend,
    Enemy,
}

/// Panchadha (fivefold compound) relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompoundRelation {
    GreatFriend,
    Friend,
    Neutral,
    Enemy,
    GreatEnemy,
}

/// The seven grahas that have relationships, in traditional order.
pub const SEVEN: [Graha; 7] = [
    Graha::Sun, Graha::Moon, Graha::Mars, Graha::Mercury,
    Graha::Jupiter, Graha::Venus, Graha::Saturn,
];

#[inline]
fn is_seven(g: Graha) -> bool {
    !g.is_chhaya()
}

/// How `of` regards `toward`, naturally. BPHS ch. 3.
///
/// Not symmetric: the Sun regards Mercury as neutral, but Mercury regards the
/// Sun as a friend.
pub fn natural(of: Graha, toward: Graha) -> Option<NaturalRelation> {
    use Graha::*;
    use NaturalRelation::{Enemy as E, Friend as F, Neutral as N};
    if of == toward || !is_seven(of) || !is_seven(toward) {
        return None;
    }
    // Row = `of`, column = `toward`, in the order Sun Moon Mars Mercury
    // Jupiter Venus Saturn. The diagonal is never read.
    const X: NaturalRelation = N;
    const TABLE: [[NaturalRelation; 7]; 7] = [
        //    Su Mo Ma Me Ju Ve Sa
        /*Su*/ [X, F, F, N, F, E, E],
        /*Mo*/ [F, X, N, F, N, N, N],
        /*Ma*/ [F, F, X, E, F, N, N],
        /*Me*/ [F, E, N, X, N, F, N],
        /*Ju*/ [F, F, F, E, X, E, N],
        /*Ve*/ [E, E, N, F, N, X, F],
        /*Sa*/ [E, E, E, F, N, F, X],
    ];
    let idx = |g: Graha| match g {
        Sun => 0, Moon => 1, Mars => 2, Mercury => 3, Jupiter => 4, Venus => 5, Saturn => 6,
        Rahu | Ketu => unreachable!("filtered above"),
    };
    Some(TABLE[idx(of)][idx(toward)])
}

/// Temporary relationship between grahas occupying `of_sign` and `toward_sign`.
///
/// The graha in the 2nd, 3rd, 4th, 10th, 11th or 12th sign from `of` is its
/// temporary friend. Anything else, including the same sign, is a temporary
/// enemy.
pub fn temporary_by_signs(of_sign: Rasi, toward_sign: Rasi) -> TemporaryRelation {
    match of_sign.houses_to(toward_sign) {
        2 | 3 | 4 | 10 | 11 | 12 => TemporaryRelation::Friend,
        _ => TemporaryRelation::Enemy,
    }
}

/// Combine natural and temporary into the compound relationship.
pub fn combine(natural: NaturalRelation, temporary: TemporaryRelation) -> CompoundRelation {
    use CompoundRelation as C;
    match (natural, temporary) {
        (NaturalRelation::Friend, TemporaryRelation::Friend) => C::GreatFriend,
        (NaturalRelation::Friend, TemporaryRelation::Enemy) => C::Neutral,
        (NaturalRelation::Neutral, TemporaryRelation::Friend) => C::Friend,
        (NaturalRelation::Neutral, TemporaryRelation::Enemy) => C::Enemy,
        (NaturalRelation::Enemy, TemporaryRelation::Friend) => C::Neutral,
        (NaturalRelation::Enemy, TemporaryRelation::Enemy) => C::GreatEnemy,
    }
}

/// Compound relationship of `of` toward `toward`, given the signs each
/// occupies in whichever chart supplies the temporary relationship.
pub fn compound_by_signs(
    of: Graha,
    toward: Graha,
    of_sign: Rasi,
    toward_sign: Rasi,
) -> Option<CompoundRelation> {
    let n = natural(of, toward)?;
    Some(combine(n, temporary_by_signs(of_sign, toward_sign)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// The natural table restated as friend/neutral/enemy lists, exactly as
    /// DESIGN.md 4.3 gives them, so a transposed matrix cell cannot hide.
    fn spec(of: Graha) -> (&'static [Graha], &'static [Graha], &'static [Graha]) {
        match of {
            Sun => (&[Moon, Mars, Jupiter], &[Mercury], &[Venus, Saturn]),
            Moon => (&[Sun, Mercury], &[Mars, Jupiter, Venus, Saturn], &[]),
            Mars => (&[Sun, Moon, Jupiter], &[Venus, Saturn], &[Mercury]),
            Mercury => (&[Sun, Venus], &[Mars, Jupiter, Saturn], &[Moon]),
            Jupiter => (&[Sun, Moon, Mars], &[Saturn], &[Mercury, Venus]),
            Venus => (&[Mercury, Saturn], &[Mars, Jupiter], &[Sun, Moon]),
            Saturn => (&[Mercury, Venus], &[Jupiter], &[Sun, Moon, Mars]),
            _ => unreachable!(),
        }
    }

    #[test]
    fn natural_table_matches_the_specification_lists() {
        for of in SEVEN {
            let (f, n, e) = spec(of);
            assert_eq!(f.len() + n.len() + e.len(), 6, "{} lists incomplete", of.name());
            for &t in f { assert_eq!(natural(of, t), Some(NaturalRelation::Friend), "{}->{}", of.name(), t.name()); }
            for &t in n { assert_eq!(natural(of, t), Some(NaturalRelation::Neutral), "{}->{}", of.name(), t.name()); }
            for &t in e { assert_eq!(natural(of, t), Some(NaturalRelation::Enemy), "{}->{}", of.name(), t.name()); }
        }
    }

    #[test]
    fn natural_is_undefined_for_self_and_the_nodes() {
        for g in SEVEN {
            assert_eq!(natural(g, g), None);
            assert_eq!(natural(g, Rahu), None);
            assert_eq!(natural(Ketu, g), None);
        }
    }

    #[test]
    fn natural_is_asymmetric_where_the_texts_say_so() {
        assert_eq!(natural(Sun, Mercury), Some(NaturalRelation::Neutral));
        assert_eq!(natural(Mercury, Sun), Some(NaturalRelation::Friend));
        assert_eq!(natural(Moon, Mercury), Some(NaturalRelation::Friend));
        assert_eq!(natural(Mercury, Moon), Some(NaturalRelation::Enemy));
    }

    #[test]
    fn temporary_friends_are_exactly_houses_2_3_4_10_11_12() {
        let from = Rasi::Mesha;
        for h in 1..=12u8 {
            let to = Rasi::from_index(h as i32 - 1);
            let expected = if [2, 3, 4, 10, 11, 12].contains(&h) {
                TemporaryRelation::Friend
            } else {
                TemporaryRelation::Enemy
            };
            assert_eq!(temporary_by_signs(from, to), expected, "house {h}");
        }
    }

    #[test]
    fn temporary_is_symmetric() {
        // h and 14-h are both in or both out of {2,3,4,10,11,12}.
        for a in Rasi::ALL {
            for b in Rasi::ALL {
                assert_eq!(temporary_by_signs(a, b), temporary_by_signs(b, a));
            }
        }
    }

    #[test]
    fn compound_follows_the_six_row_table() {
        use CompoundRelation as C;
        use NaturalRelation as N;
        use TemporaryRelation as T;
        assert_eq!(combine(N::Friend, T::Friend), C::GreatFriend);
        assert_eq!(combine(N::Friend, T::Enemy), C::Neutral);
        assert_eq!(combine(N::Neutral, T::Friend), C::Friend);
        assert_eq!(combine(N::Neutral, T::Enemy), C::Enemy);
        assert_eq!(combine(N::Enemy, T::Friend), C::Neutral);
        assert_eq!(combine(N::Enemy, T::Enemy), C::GreatEnemy);
    }
}
