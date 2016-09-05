//! Parsers for applying parsers multiple times

/// `separated_list!(I -> IResult<I,T>, I -> IResult<I,O>) => I -> IResult<I, Vec<O>>`
/// separated_list(sep, X) returns Vec<X>
#[macro_export]
macro_rules! separated_list(
  ($i:expr, $sep:ident!( $($args:tt)* ), $submac:ident!( $($args2:tt)* )) => (
    {
      let mut res   = ::std::vec::Vec::new();
      let mut input = $i;

      // get the first element
      match $submac!(input, $($args2)*) {
        $crate::IResult::Error(_)      => $crate::IResult::Done(input, ::std::vec::Vec::new()),
        $crate::IResult::Incomplete(i) => $crate::IResult::Incomplete(i),
        $crate::IResult::Done(i,o)     => {
          if i.len() == input.len() {
            $crate::IResult::Error(error_position!($crate::ErrorKind::SeparatedList,input))
          } else {
            res.push(o);
            input = i;

            loop {
              // get the separator first
              if let $crate::IResult::Done(i2,_) = $sep!(input, $($args)*) {
                if i2.len() == input.len() {
                  break;
                }

                // get the element next
                if let $crate::IResult::Done(i3,o3) = $submac!(i2, $($args2)*) {
                  if i3.len() == i2.len() {
                    break;
                  }
                  res.push(o3);
                  input = i3;
                } else {
                  break;
                }
              } else {
                break;
              }
            }
            $crate::IResult::Done(input, res)
          }
        },
      }
    }
  );
  ($i:expr, $submac:ident!( $($args:tt)* ), $g:expr) => (
    separated_list!($i, $submac!($($args)*), call!($g));
  );
  ($i:expr, $f:expr, $submac:ident!( $($args:tt)* )) => (
    separated_list!($i, call!($f), $submac!($($args)*));
  );
  ($i:expr, $f:expr, $g:expr) => (
    separated_list!($i, call!($f), call!($g));
  );
);

/// `separated_nonempty_list!(I -> IResult<I,T>, I -> IResult<I,O>) => I -> IResult<I, Vec<O>>`
/// separated_nonempty_list(sep, X) returns Vec<X>
#[macro_export]
macro_rules! separated_nonempty_list(
  ($i:expr, $sep:ident!( $($args:tt)* ), $submac:ident!( $($args2:tt)* )) => (
    {
      let mut res   = ::std::vec::Vec::new();
      let mut input = $i;

      // get the first element
      match $submac!(input, $($args2)*) {
        $crate::IResult::Error(a)      => $crate::IResult::Error(a),
        $crate::IResult::Incomplete(i) => $crate::IResult::Incomplete(i),
        $crate::IResult::Done(i,o)     => {
          if i.len() == input.len() {
            $crate::IResult::Error(error_position!($crate::ErrorKind::SeparatedNonEmptyList,input))
          } else {
            res.push(o);
            input = i;

            loop {
              if let $crate::IResult::Done(i2,_) = $sep!(input, $($args)*) {
                if i2.len() == input.len() {
                  break;
                }

                if let $crate::IResult::Done(i3,o3) = $submac!(i2, $($args2)*) {
                  if i3.len() == i2.len() {
                    break;
                  }
                  res.push(o3);
                  input = i3;
                } else {
                  break;
                }
              } else {
                break;
              }
            }
            $crate::IResult::Done(input, res)
          }
        },
      }
    }
  );
  ($i:expr, $submac:ident!( $($args:tt)* ), $g:expr) => (
    separated_nonempty_list!($i, $submac!($($args)*), call!($g));
  );
  ($i:expr, $f:expr, $submac:ident!( $($args:tt)* )) => (
    separated_nonempty_list!($i, call!($f), $submac!($($args)*));
  );
  ($i:expr, $f:expr, $g:expr) => (
    separated_nonempty_list!($i, call!($f), call!($g));
  );
);

/// `many0!(I -> IResult<I,O>) => I -> IResult<I, Vec<O>>`
/// Applies the parser 0 or more times and returns the list of results in a Vec
///
/// the embedded parser may return Incomplete
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # fn main() {
///  named!(multi<&[u8], Vec<&[u8]> >, many0!( tag!( "abcd" ) ) );
///
///  let a = b"abcdabcdefgh";
///  let b = b"azerty";
///
///  let res = vec![&b"abcd"[..], &b"abcd"[..]];
///  assert_eq!(multi(&a[..]), Done(&b"efgh"[..], res));
///  assert_eq!(multi(&b[..]), Done(&b"azerty"[..], Vec::new()));
/// # }
/// ```
/// 0 or more
#[macro_export]
macro_rules! many0(
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
    {
      use $crate::InputLength;

      let ret;
      let mut res   = ::std::vec::Vec::new();
      let mut input = $i;

      loop {
        if input.input_len() == 0 {
          ret = $crate::IResult::Done(input, res);
          break;
        }

        match $submac!(input, $($args)*) {
          $crate::IResult::Error(_)                            => {
            ret = $crate::IResult::Done(input, res);
            break;
          },
          $crate::IResult::Incomplete($crate::Needed::Unknown) => {
            ret = $crate::IResult::Incomplete($crate::Needed::Unknown);
            break;
          },
          $crate::IResult::Incomplete($crate::Needed::Size(i)) => {
            let size = i + ($i).input_len() - input.input_len();
            ret = $crate::IResult::Incomplete($crate::Needed::Size(size));
            break;
          },
          $crate::IResult::Done(i, o)                          => {
            // loop trip must always consume (otherwise infinite loops)
            if i == input {
              ret = $crate::IResult::Error(error_position!($crate::ErrorKind::Many0,input));
              break;
            }

            res.push(o);
            input = i;
          }
        }
      }

      ret
    }
  );
  ($i:expr, $f:expr) => (
    many0!($i, call!($f));
  );
);

/// `many1!(I -> IResult<I,O>) => I -> IResult<I, Vec<O>>`
/// Applies the parser 1 or more times and returns the list of results in a Vec
///
/// the embedded parser may return Incomplete
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::{Done, Error};
/// # #[cfg(feature = "verbose-errors")]
/// # use nom::Err::Position;
/// # use nom::ErrorKind;
/// # fn main() {
///  named!(multi<&[u8], Vec<&[u8]> >, many1!( tag!( "abcd" ) ) );
///
///  let a = b"abcdabcdefgh";
///  let b = b"azerty";
///
///  let res = vec![&b"abcd"[..], &b"abcd"[..]];
///  assert_eq!(multi(&a[..]), Done(&b"efgh"[..], res));
///  assert_eq!(multi(&b[..]), Error(error_position!(ErrorKind::Many1,&b[..])));
/// # }
/// ```
#[macro_export]
macro_rules! many1(
  ($i:expr, $submac:ident!( $($args:tt)* )) => (
    {
      use $crate::InputLength;
      match $submac!($i, $($args)*) {
        $crate::IResult::Error(_)      => $crate::IResult::Error(
          error_position!($crate::ErrorKind::Many1,$i)
        ),
        $crate::IResult::Incomplete(i) => $crate::IResult::Incomplete(i),
        $crate::IResult::Done(i1,o1)   => {
          if i1.input_len() == 0 {
            $crate::IResult::Done(i1,vec![o1])
          } else {

            let mut res    = ::std::vec::Vec::with_capacity(4);
            res.push(o1);
            let mut input  = i1;
            let mut incomplete: ::std::option::Option<$crate::Needed> =
              ::std::option::Option::None;
            loop {
              if input.input_len() == 0 {
                break;
              }
              match $submac!(input, $($args)*) {
                $crate::IResult::Error(_)                    => {
                  break;
                },
                $crate::IResult::Incomplete($crate::Needed::Unknown) => {
                  incomplete = ::std::option::Option::Some($crate::Needed::Unknown);
                  break;
                },
                $crate::IResult::Incomplete($crate::Needed::Size(i)) => {
                  incomplete = ::std::option::Option::Some(
                    $crate::Needed::Size(i + ($i).input_len() - input.input_len())
                  );
                  break;
                },
                $crate::IResult::Done(i, o) => {
                  if i.input_len() == input.input_len() {
                    break;
                  }
                  res.push(o);
                  input = i;
                }
              }
            }

            match incomplete {
              ::std::option::Option::Some(i) => $crate::IResult::Incomplete(i),
              ::std::option::Option::None    => $crate::IResult::Done(input, res)
            }
          }
        }
      }
    }
  );
  ($i:expr, $f:expr) => (
    many1!($i, call!($f));
  );
);

/// `many_m_n!(usize, usize, I -> IResult<I,O>) => I -> IResult<I, Vec<O>>`
/// Applies the parser between m and n times (n included) and returns the list of
/// results in a Vec
///
/// the embedded parser may return Incomplete
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::{Done, Error};
/// # #[cfg(feature = "verbose-errors")]
/// # use nom::Err::Position;
/// # use nom::ErrorKind;
/// # fn main() {
///  named!(multi<&[u8], Vec<&[u8]> >, many_m_n!(2, 4, tag!( "abcd" ) ) );
///
///  let a = b"abcdefgh";
///  let b = b"abcdabcdefgh";
///  let c = b"abcdabcdabcdabcdabcdefgh";
///
///  assert_eq!(multi(&a[..]),Error(error_position!(ErrorKind::ManyMN,&a[..])));
///  let res = vec![&b"abcd"[..], &b"abcd"[..]];
///  assert_eq!(multi(&b[..]), Done(&b"efgh"[..], res));
///  let res2 = vec![&b"abcd"[..], &b"abcd"[..], &b"abcd"[..], &b"abcd"[..]];
///  assert_eq!(multi(&c[..]), Done(&b"abcdefgh"[..], res2));
/// # }
/// ```
#[macro_export]
macro_rules! many_m_n(
  ($i:expr, $m:expr, $n: expr, $submac:ident!( $($args:tt)* )) => (
    {
      use $crate::InputLength;
      let mut res          = ::std::vec::Vec::with_capacity($m);
      let mut input        = $i;
      let mut count: usize = 0;
      let mut err          = false;
      let mut incomplete: ::std::option::Option<$crate::Needed> = ::std::option::Option::None;
      loop {
        if count == $n { break }
        match $submac!(input, $($args)*) {
          $crate::IResult::Done(i, o) => {
            // do not allow parsers that do not consume input (causes infinite loops)
            if i.input_len() == input.input_len() {
              break;
            }
            res.push(o);
            input  = i;
            count += 1;
          }
          $crate::IResult::Error(_)                    => {
            err = true;
            break;
          },
          $crate::IResult::Incomplete($crate::Needed::Unknown) => {
            incomplete = ::std::option::Option::Some($crate::Needed::Unknown);
            break;
          },
          $crate::IResult::Incomplete($crate::Needed::Size(i)) => {
            incomplete = ::std::option::Option::Some(
              $crate::Needed::Size(i + ($i).input_len() - input.input_len())
            );
            break;
          },
        }
        if input.input_len() == 0 {
          break;
        }
      }

      if count < $m {
        if err {
          $crate::IResult::Error(error_position!($crate::ErrorKind::ManyMN,$i))
        } else {
          match incomplete {
            ::std::option::Option::Some(i) => $crate::IResult::Incomplete(i),
            ::std::option::Option::None    => $crate::IResult::Incomplete(
              $crate::Needed::Unknown
            )
          }
        }
      } else {
        match incomplete {
          ::std::option::Option::Some(i) => $crate::IResult::Incomplete(i),
          ::std::option::Option::None    => $crate::IResult::Done(input, res)
        }
      }
    }
  );
  ($i:expr, $m:expr, $n: expr, $f:expr) => (
    many_m_n!($i, $m, $n, call!($f));
  );
);

/// `count!(I -> IResult<I,O>, nb) => I -> IResult<I, Vec<O>>`
/// Applies the child parser a specified number of times
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::{Done,Error};
/// # #[cfg(feature = "verbose-errors")]
/// # use nom::Err::Position;
/// # use nom::ErrorKind;
/// # fn main() {
///  named!(counter< Vec<&[u8]> >, count!( tag!( "abcd" ), 2 ) );
///
///  let a = b"abcdabcdabcdef";
///  let b = b"abcdefgh";
///  let res = vec![&b"abcd"[..], &b"abcd"[..]];
///
///  assert_eq!(counter(&a[..]), Done(&b"abcdef"[..], res));
///  assert_eq!(counter(&b[..]), Error(error_position!(ErrorKind::Count, &b[..])));
/// # }
/// ```
///
#[macro_export]
macro_rules! count(
  ($i:expr, $submac:ident!( $($args:tt)* ), $count: expr) => (
    {
      let ret;
      let mut input = $i;
      let mut res   = ::std::vec::Vec::with_capacity($count);

      loop {
        if res.len() == $count {
          ret = $crate::IResult::Done(input, res);
          break;
        }

        match $submac!(input, $($args)*) {
          $crate::IResult::Done(i,o) => {
            res.push(o);
            input = i;
          },
          $crate::IResult::Error(_)  => {
            ret = $crate::IResult::Error(error_position!($crate::ErrorKind::Count,$i));
            break;
          },
          $crate::IResult::Incomplete(_) => {
            ret = $crate::IResult::Incomplete($crate::Needed::Unknown);
            break;
          }
        }
      }

      ret
    }
  );
  ($i:expr, $f:expr, $count: expr) => (
    count!($i, call!($f), $count);
  );
);

/// `count_fixed!(O, I -> IResult<I,O>, nb) => I -> IResult<I, [O; nb]>`
/// Applies the child parser a fixed number of times and returns a fixed size array
/// The type must be specified and it must be `Copy`
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::{Done,Error};
/// # #[cfg(feature = "verbose-errors")]
/// # use nom::Err::Position;
/// # use nom::ErrorKind;
/// # fn main() {
///  named!(counter< [&[u8]; 2] >, count_fixed!( &[u8], tag!( "abcd" ), 2 ) );
///  // can omit the type specifier if returning slices
///  // named!(counter< [&[u8]; 2] >, count_fixed!( tag!( "abcd" ), 2 ) );
///
///  let a = b"abcdabcdabcdef";
///  let b = b"abcdefgh";
///  let res = [&b"abcd"[..], &b"abcd"[..]];
///
///  assert_eq!(counter(&a[..]), Done(&b"abcdef"[..], res));
///  assert_eq!(counter(&b[..]), Error(error_position!(ErrorKind::Count, &b[..])));
/// # }
/// ```
///
#[macro_export]
macro_rules! count_fixed (
  ($i:expr, $typ:ty, $submac:ident!( $($args:tt)* ), $count: expr) => (
    {
      let ret;
      let mut input = $i;
      // `$typ` must be Copy, and thus having no destructor, this is panic safe
      let mut res: [$typ; $count] = unsafe{[::std::mem::uninitialized(); $count as usize]};
      let mut cnt: usize = 0;

      loop {
        if cnt == $count {
          ret = $crate::IResult::Done(input, res); break;
        }

        match $submac!(input, $($args)*) {
          $crate::IResult::Done(i,o) => {
            res[cnt] = o;
            cnt += 1;
            input = i;
          },
          $crate::IResult::Error(_)  => {
            ret = $crate::IResult::Error(error_position!($crate::ErrorKind::Count,$i));
            break;
          },
          $crate::IResult::Incomplete(_) => {
            ret = $crate::IResult::Incomplete($crate::Needed::Unknown);
            break;
          }
        }
      }

      ret
    }
  );
  ($i:expr, $typ: ty, $f:ident, $count: expr) => (
    count_fixed!($i, $typ, call!($f), $count);
  );
);

/// `length_value!(I -> IResult<I, nb>, I -> IResult<I,O>) => I -> IResult<I, Vec<O>>`
/// gets a number from the first parser, then applies the second parser that many times
#[macro_export]
macro_rules! length_value(
  ($i:expr, $f:expr, $g:expr) => (
    {
      match $f($i) {
        $crate::IResult::Error(a)      => $crate::IResult::Error(a),
        $crate::IResult::Incomplete(x) => $crate::IResult::Incomplete(x),
        $crate::IResult::Done(inum, onum)   => {
          let ret;
          let length_token = $i.len() - inum.len();
          let mut input    = inum;
          let mut res      = ::std::vec::Vec::new();

          loop {
            if res.len() == onum as usize {
              ret = $crate::IResult::Done(input, res); break;
            }

            match $g(input) {
              $crate::IResult::Done(iparse, oparse) => {
                res.push(oparse);
                input = iparse;
              },
              $crate::IResult::Error(_)      => {
                ret = $crate::IResult::Error(error_position!($crate::ErrorKind::LengthValue,$i));
                break;
              },
              $crate::IResult::Incomplete(a) => {
                ret = match a {
                  $crate::Needed::Unknown      => $crate::IResult::Incomplete(
                    $crate::Needed::Unknown
                  ),
                  $crate::Needed::Size(length) => $crate::IResult::Incomplete(
                    $crate::Needed::Size(length_token + onum as usize * length)
                  )
                };
                break;
              }
            }
          }

          ret
        }
      }
    }
  );
  ($i:expr, $f:expr, $g:expr, $length:expr) => (
    {
      match $f($i) {
        $crate::IResult::Error(a)      => $crate::IResult::Error(a),
        $crate::IResult::Incomplete(x) => $crate::IResult::Incomplete(x),
        $crate::IResult::Done(inum, onum)   => {
          let ret;
          let length_token = $i.len() - inum.len();
          let mut input    = inum;
          let mut res      = ::std::vec::Vec::new();

          loop {
            if res.len() == onum as usize {
              ret = $crate::IResult::Done(input, res); break;
            }

            match $g(input) {
              $crate::IResult::Done(iparse, oparse) => {
                res.push(oparse);
                input = iparse;
              },
              $crate::IResult::Error(_)      => {
                ret = $crate::IResult::Error(error_position!($crate::ErrorKind::LengthValue,$i));
                break;
              },
              $crate::IResult::Incomplete(a) => {
                ret = match a {
                  $crate::Needed::Unknown => $crate::IResult::Incomplete(
                    $crate::Needed::Unknown
                  ),
                  $crate::Needed::Size(_) => $crate::IResult::Incomplete(
                    $crate::Needed::Size(length_token + onum as usize * $length)
                  )
                };
                break;
              }
            }
          }

          ret
        }
      }
    }
  );
);

/// `fold_many0!(I -> IResult<I,O>, R, Fn(R, O) -> R) => I -> IResult<I, R>`
/// Applies the parser 0 or more times and folds the list of return values
///
/// the embedded parser may return Incomplete
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::Done;
/// # fn main() {
///  named!(multi<&[u8], Vec<&[u8]> >,
///    fold_many0!( tag!( "abcd" ), Vec::new(), |mut acc: Vec<_>, item| {
///      acc.push(item);
///      acc
///  }));
///
///  let a = b"abcdabcdefgh";
///  let b = b"azerty";
///
///  let res = vec![&b"abcd"[..], &b"abcd"[..]];
///  assert_eq!(multi(&a[..]), Done(&b"efgh"[..], res));
///  assert_eq!(multi(&b[..]), Done(&b"azerty"[..], Vec::new()));
/// # }
/// ```
/// 0 or more
#[macro_export]
macro_rules! fold_many0(
  ($i:expr, $submac:ident!( $($args:tt)* ), $init:expr, $f:expr) => (
    {
      use $crate::InputLength;
      let ret;
      let f         = $f;
      let mut res   = $init;
      let mut input = $i;

      loop {
        if input.input_len() == 0 {
          ret = $crate::IResult::Done(input, res);
          break;
        }

        match $submac!(input, $($args)*) {
          $crate::IResult::Error(_)                            => {
            ret = $crate::IResult::Done(input, res);
            break;
          },
          $crate::IResult::Incomplete($crate::Needed::Unknown) => {
            ret = $crate::IResult::Incomplete($crate::Needed::Unknown);
            break;
          },
          $crate::IResult::Incomplete($crate::Needed::Size(i)) => {
            let size = i + ($i).input_len() - input.input_len();
            ret = $crate::IResult::Incomplete($crate::Needed::Size(size));
            break;
          },
          $crate::IResult::Done(i, o)                          => {
            // loop trip must always consume (otherwise infinite loops)
            if i == input {
              ret = $crate::IResult::Error(
                error_position!($crate::ErrorKind::Many0,input)
              );
              break;
            }

            res = f(res, o);
            input = i;
          }
        }
      }

      ret
    }
  );
  ($i:expr, $f:expr, $init:expr, $fold_f:expr) => (
    fold_many0!($i, call!($f), $init, $fold_f);
  );
);

/// `fold_many1!(I -> IResult<I,O>, R, Fn(R, O) -> R) => I -> IResult<I, R>`
/// Applies the parser 1 or more times and folds the list of return values
///
/// the embedded parser may return Incomplete
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::{Done, Error};
/// # #[cfg(feature = "verbose-errors")]
/// # use nom::Err::Position;
/// # use nom::ErrorKind;
/// # fn main() {
///  named!(multi<&[u8], Vec<&[u8]> >,
///    fold_many1!( tag!( "abcd" ), Vec::new(), |mut acc: Vec<_>, item| {
///      acc.push(item);
///      acc
///  }));
///
///  let a = b"abcdabcdefgh";
///  let b = b"azerty";
///
///  let res = vec![&b"abcd"[..], &b"abcd"[..]];
///  assert_eq!(multi(&a[..]), Done(&b"efgh"[..], res));
///  assert_eq!(multi(&b[..]), Error(error_position!(ErrorKind::Many1,&b[..])));
/// # }
/// ```
#[macro_export]
macro_rules! fold_many1(
  ($i:expr, $submac:ident!( $($args:tt)* ), $init:expr, $f:expr) => (
    {
      use $crate::InputLength;
      match $submac!($i, $($args)*) {
        $crate::IResult::Error(_)      => $crate::IResult::Error(
          error_position!($crate::ErrorKind::Many1,$i)
        ),
        $crate::IResult::Incomplete(i) => $crate::IResult::Incomplete(i),
        $crate::IResult::Done(i1,o1)   => {
          let acc = $init;
          let f = $f;
          if i1.len() == 0 {
            let acc = f(acc, o1);
            $crate::IResult::Done(i1,acc)
          } else {
            let mut acc = f(acc, o1);
            let mut input  = i1;
            let mut incomplete: ::std::option::Option<$crate::Needed> =
              ::std::option::Option::None;
            loop {
              if input.input_len() == 0 {
                break;
              }
              match $submac!(input, $($args)*) {
                $crate::IResult::Error(_)                    => {
                  break;
                },
                $crate::IResult::Incomplete($crate::Needed::Unknown) => {
                  incomplete = ::std::option::Option::Some($crate::Needed::Unknown);
                  break;
                },
                $crate::IResult::Incomplete($crate::Needed::Size(i)) => {
                  incomplete = ::std::option::Option::Some(
                    $crate::Needed::Size(i + ($i).input_len() - input.input_len())
                  );
                  break;
                },
                $crate::IResult::Done(i, o) => {
                  if i.input_len() == input.input_len() {
                    break;
                  }
                  acc = f(acc, o);
                  input = i;
                }
              }
            }

            match incomplete {
              ::std::option::Option::Some(i) => $crate::IResult::Incomplete(i),
              ::std::option::Option::None    => $crate::IResult::Done(input, acc)
            }
          }
        }
      }
    }
  );
  ($i:expr, $f:expr, $init:expr, $fold_f:expr) => (
    fold_many1!($i, call!($f), $init, $fold_f);
  );
);

/// `fold_many_m_n!(usize, usize, I -> IResult<I,O>, R, Fn(R, O) -> R) => I -> IResult<I, R>`
/// Applies the parser between m and n times (n included) and folds the list of return value
///
/// the embedded parser may return Incomplete
///
/// ```
/// # #[macro_use] extern crate nom;
/// # use nom::IResult::{Done, Error};
/// # #[cfg(feature = "verbose-errors")]
/// # use nom::Err::Position;
/// # use nom::ErrorKind;
/// # fn main() {
///  named!(multi<&[u8], Vec<&[u8]> >,
///    fold_many_m_n!(2, 4, tag!( "abcd" ), Vec::new(), |mut acc: Vec<_>, item| {
///      acc.push(item);
///      acc
///  }));
///
///  let a = b"abcdefgh";
///  let b = b"abcdabcdefgh";
///  let c = b"abcdabcdabcdabcdabcdefgh";
///
///  assert_eq!(multi(&a[..]),Error(error_position!(ErrorKind::ManyMN,&a[..])));
///  let res = vec![&b"abcd"[..], &b"abcd"[..]];
///  assert_eq!(multi(&b[..]), Done(&b"efgh"[..], res));
///  let res2 = vec![&b"abcd"[..], &b"abcd"[..], &b"abcd"[..], &b"abcd"[..]];
///  assert_eq!(multi(&c[..]), Done(&b"abcdefgh"[..], res2));
/// # }
/// ```
#[macro_export]
macro_rules! fold_many_m_n(
  ($i:expr, $m:expr, $n: expr, $submac:ident!( $($args:tt)* ), $init:expr, $f:expr) => (
    {
      use $crate::InputLength;
      let mut acc          = $init;
      let     f            = $f;
      let mut input        = $i;
      let mut count: usize = 0;
      let mut err          = false;
      let mut incomplete: ::std::option::Option<$crate::Needed> = ::std::option::Option::None;
      loop {
        if count == $n { break }
        match $submac!(input, $($args)*) {
          $crate::IResult::Done(i, o) => {
            // do not allow parsers that do not consume input (causes infinite loops)
            if i.input_len() == input.input_len() {
              break;
            }
            acc = f(acc, o);
            input  = i;
            count += 1;
          }
          $crate::IResult::Error(_)                    => {
            err = true;
            break;
          },
          $crate::IResult::Incomplete($crate::Needed::Unknown) => {
            incomplete = ::std::option::Option::Some($crate::Needed::Unknown);
            break;
          },
          $crate::IResult::Incomplete($crate::Needed::Size(i)) => {
            incomplete = ::std::option::Option::Some(
              $crate::Needed::Size(i + ($i).input_len() - input.input_len())
            );
            break;
          },
        }
        if input.input_len() == 0 {
          break;
        }
      }

      if count < $m {
        if err {
          $crate::IResult::Error(error_position!($crate::ErrorKind::ManyMN,$i))
        } else {
          match incomplete {
            ::std::option::Option::Some(i) => $crate::IResult::Incomplete(i),
            ::std::option::Option::None    => $crate::IResult::Incomplete($crate::Needed::Unknown)
          }
        }
      } else {
        match incomplete {
          ::std::option::Option::Some(i) => $crate::IResult::Incomplete(i),
          ::std::option::Option::None    => $crate::IResult::Done(input, acc)
        }
      }
    }
  );
  ($i:expr, $m:expr, $n: expr, $f:expr, $init:expr, $fold_f:expr) => (
    fold_many_m_n!($i, $m, $n, call!($f), $init, $fold_f);
  );
);

#[cfg(test)]
mod tests {
  use internal::{Needed,IResult};

  use internal::IResult::*;
  use util::ErrorKind;
  use nom::{be_u8,be_u16,le_u16};

  // reproduce the tag and take macros, because of module import order
  macro_rules! tag (
    ($i:expr, $inp: expr) => (
      {
        #[inline(always)]
        fn as_bytes<T: $crate::AsBytes>(b: &T) -> &[u8] {
          b.as_bytes()
        }

        let expected = $inp;
        let bytes    = as_bytes(&expected);

        tag_bytes!($i,bytes)
      }
    );
  );

  macro_rules! tag_bytes (
    ($i:expr, $bytes: expr) => (
      {
        use std::cmp::min;
        let len = $i.len();
        let blen = $bytes.len();
        let m   = min(len, blen);
        let reduced = &$i[..m];
        let b       = &$bytes[..m];

        let res: $crate::IResult<_,_> = if reduced != b {
          $crate::IResult::Error(error_position!($crate::ErrorKind::Tag, $i))
        } else if m < blen {
          $crate::IResult::Incomplete($crate::Needed::Size(blen))
        } else {
          $crate::IResult::Done(&$i[blen..], reduced)
        };
        res
      }
    );
  );

  macro_rules! take(
    ($i:expr, $count:expr) => (
      {
        let cnt = $count as usize;
        let res:$crate::IResult<&[u8],&[u8]> = if $i.len() < cnt {
          $crate::IResult::Incomplete($crate::Needed::Size(cnt))
        } else {
          $crate::IResult::Done(&$i[cnt..],&$i[0..cnt])
        };
        res
      }
    )
  );

  #[test]
  fn separated_list() {
    named!(multi<&[u8],Vec<&[u8]> >, separated_list!(tag!(","), tag!("abcd")));
    named!(multi_empty<&[u8],Vec<&[u8]> >, separated_list!(tag!(","), tag!("")));

    let a = &b"abcdef"[..];
    let b = &b"abcd,abcdef"[..];
    let c = &b"azerty"[..];
    let d = &b",,abc"[..];
    let e = &b"abcd,abcd,ef"[..];

    let res1 = vec![&b"abcd"[..]];
    assert_eq!(multi(a), Done(&b"ef"[..], res1));
    let res2 = vec![&b"abcd"[..], &b"abcd"[..]];
    assert_eq!(multi(b), Done(&b"ef"[..], res2));
    assert_eq!(multi(c), Done(&b"azerty"[..], Vec::new()));
    assert_eq!(multi_empty(d), Error(error_position!(ErrorKind::SeparatedList, d)));
    //let res3 = vec![&b""[..], &b""[..], &b""[..]];
    //assert_eq!(multi_empty(d), Done(&b"abc"[..], res3));
    let res4 = vec![&b"abcd"[..], &b"abcd"[..]];
    assert_eq!(multi(e), Done(&b",ef"[..], res4));
  }

  #[test]
  fn separated_nonempty_list() {
    named!(multi<&[u8],Vec<&[u8]> >, separated_nonempty_list!(tag!(","), tag!("abcd")));

    let a = &b"abcdef"[..];
    let b = &b"abcd,abcdef"[..];
    let c = &b"azerty"[..];
    let d = &b"abcd,abcd,ef"[..];

    let res1 = vec![&b"abcd"[..]];
    assert_eq!(multi(a), Done(&b"ef"[..], res1));
    let res2 = vec![&b"abcd"[..], &b"abcd"[..]];
    assert_eq!(multi(b), Done(&b"ef"[..], res2));
    assert_eq!(multi(c), Error(error_position!(ErrorKind::Tag,c)));
    let res3 = vec![&b"abcd"[..], &b"abcd"[..]];
    assert_eq!(multi(d), Done(&b",ef"[..], res3));
  }

  #[test]
  fn many0() {
    named!( tag_abcd, tag!("abcd") );
    named!( tag_empty, tag!("") );
    named!( multi<&[u8],Vec<&[u8]> >, many0!(tag_abcd) );
    named!( multi_empty<&[u8],Vec<&[u8]> >, many0!(tag_empty) );

    assert_eq!(multi(&b"abcdef"[..]), Done(&b"ef"[..], vec![&b"abcd"[..]]));
    assert_eq!(multi(&b"abcdabcdefgh"[..]), Done(&b"efgh"[..], vec![&b"abcd"[..], &b"abcd"[..]]));
    assert_eq!(multi(&b"azerty"[..]), Done(&b"azerty"[..], Vec::new()));
    assert_eq!(multi(&b"abcdab"[..]), Incomplete(Needed::Size(8)));
    assert_eq!(multi(&b"abcd"[..]), Done(&b""[..], vec![&b"abcd"[..]]));
    assert_eq!(multi(&b""[..]), Done(&b""[..], Vec::new()));
    assert_eq!(multi_empty(&b"abcdef"[..]), Error(error_position!(ErrorKind::Many0, &b"abcdef"[..])));
  }

  #[cfg(feature = "nightly")]
  use test::Bencher;

  #[cfg(feature = "nightly")]
  #[bench]
  fn many0_bench(b: &mut Bencher) {
    named!(multi<&[u8],Vec<&[u8]> >, many0!(tag!("abcd")));
    b.iter(|| {
      multi(&b"abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd"[..])
    });
  }

  #[test]
  fn many1() {
    named!(multi<&[u8],Vec<&[u8]> >, many1!(tag!("abcd")));

    let a = &b"abcdef"[..];
    let b = &b"abcdabcdefgh"[..];
    let c = &b"azerty"[..];
    let d = &b"abcdab"[..];

    let res1 = vec![&b"abcd"[..]];
    assert_eq!(multi(a), Done(&b"ef"[..], res1));
    let res2 = vec![&b"abcd"[..], &b"abcd"[..]];
    assert_eq!(multi(b), Done(&b"efgh"[..], res2));
    assert_eq!(multi(c), Error(error_position!(ErrorKind::Many1,c)));
    assert_eq!(multi(d), Incomplete(Needed::Size(8)));
  }

  #[test]
  fn infinite_many() {
    fn tst(input: &[u8]) -> IResult<&[u8], &[u8]> {
      println!("input: {:?}", input);
      Error(error_position!(ErrorKind::Custom(0),input))
    }

    // should not go into an infinite loop
    named!(multi0<&[u8],Vec<&[u8]> >, many0!(tst));
    let a = &b"abcdef"[..];
    assert_eq!(multi0(a), Done(a, Vec::new()));

    named!(multi1<&[u8],Vec<&[u8]> >, many1!(tst));
    let a = &b"abcdef"[..];
    assert_eq!(multi1(a), Error(error_position!(ErrorKind::Many1,a)));
  }

  #[test]
  fn many_m_n() {
    named!(multi<&[u8],Vec<&[u8]> >, many_m_n!(2, 4, tag!("Abcd")));

    let a = &b"Abcdef"[..];
    let b = &b"AbcdAbcdefgh"[..];
    let c = &b"AbcdAbcdAbcdAbcdefgh"[..];
    let d = &b"AbcdAbcdAbcdAbcdAbcdefgh"[..];
    let e = &b"AbcdAb"[..];

    assert_eq!(multi(a), Error(error_position!(ErrorKind::ManyMN,a)));
    let res1 = vec![&b"Abcd"[..], &b"Abcd"[..]];
    assert_eq!(multi(b), Done(&b"efgh"[..], res1));
    let res2 = vec![&b"Abcd"[..], &b"Abcd"[..], &b"Abcd"[..], &b"Abcd"[..]];
    assert_eq!(multi(c), Done(&b"efgh"[..], res2));
    let res3 = vec![&b"Abcd"[..], &b"Abcd"[..], &b"Abcd"[..], &b"Abcd"[..]];
    assert_eq!(multi(d), Done(&b"Abcdefgh"[..], res3));
    assert_eq!(multi(e), Incomplete(Needed::Size(8)));
  }

  #[test]
  fn count() {
    const TIMES: usize = 2;
    named!( tag_abc, tag!("abc") );
    named!( cnt_2<&[u8], Vec<&[u8]> >, count!(tag_abc, TIMES ) );

    assert_eq!(cnt_2(&b"abcabcabcdef"[..]), Done(&b"abcdef"[..], vec![&b"abc"[..], &b"abc"[..]]));
    assert_eq!(cnt_2(&b"ab"[..]), Incomplete(Needed::Unknown));
    assert_eq!(cnt_2(&b"abcab"[..]), Incomplete(Needed::Unknown));
    assert_eq!(cnt_2(&b"xxx"[..]), Error(error_position!(ErrorKind::Count, &b"xxx"[..])));
    assert_eq!(cnt_2(&b"xxxabcabcdef"[..]), Error(error_position!(ErrorKind::Count, &b"xxxabcabcdef"[..])));
    assert_eq!(cnt_2(&b"abcxxxabcdef"[..]), Error(error_position!(ErrorKind::Count, &b"abcxxxabcdef"[..])));
  }

  #[test]
  fn count_zero() {
    const TIMES: usize = 0;
    named!( tag_abc, tag!("abc") );
    named!( counter_2<&[u8], Vec<&[u8]> >, count!(tag_abc, TIMES ) );

    let done = &b"abcabcabcdef"[..];
    let parsed_done = Vec::new();
    let rest = done;
    let incomplete_1 = &b"ab"[..];
    let parsed_incompl_1 = Vec::new();
    let incomplete_2 = &b"abcab"[..];
    let parsed_incompl_2 = Vec::new();
    let error = &b"xxx"[..];
    let error_remain = &b"xxx"[..];
    let parsed_err = Vec::new();
    let error_1 = &b"xxxabcabcdef"[..];
    let parsed_err_1 = Vec::new();
    let error_1_remain = &b"xxxabcabcdef"[..];
    let error_2 = &b"abcxxxabcdef"[..];
    let parsed_err_2 = Vec::new();
    let error_2_remain = &b"abcxxxabcdef"[..];

    assert_eq!(counter_2(done), Done(rest, parsed_done));
    assert_eq!(counter_2(incomplete_1), Done(incomplete_1, parsed_incompl_1));
    assert_eq!(counter_2(incomplete_2), Done(incomplete_2, parsed_incompl_2));
    assert_eq!(counter_2(error), Done(error_remain, parsed_err));
    assert_eq!(counter_2(error_1), Done(error_1_remain, parsed_err_1));
    assert_eq!(counter_2(error_2), Done(error_2_remain, parsed_err_2));
  }

  #[test]
  fn count_fixed() {
    const TIMES: usize = 2;
    named!( tag_abc, tag!("abc") );
    named!( cnt_2<&[u8], [&[u8]; TIMES] >, count_fixed!(&[u8], tag_abc, TIMES ) );

    assert_eq!(cnt_2(&b"abcabcabcdef"[..]), Done(&b"abcdef"[..], [&b"abc"[..], &b"abc"[..]]));
    assert_eq!(cnt_2(&b"ab"[..]), Incomplete(Needed::Unknown));
    assert_eq!(cnt_2(&b"abcab"[..]), Incomplete(Needed::Unknown));
    assert_eq!(cnt_2(&b"xxx"[..]), Error(error_position!(ErrorKind::Count, &b"xxx"[..])));
    assert_eq!(cnt_2(&b"xxxabcabcdef"[..]), Error(error_position!(ErrorKind::Count, &b"xxxabcabcdef"[..])));
    assert_eq!(cnt_2(&b"abcxxxabcdef"[..]), Error(error_position!(ErrorKind::Count, &b"abcxxxabcdef"[..])));
  }

  #[allow(dead_code)]
  pub fn compile_count_fixed(input: &[u8]) -> IResult<&[u8], ()> {
    chain!(input,
      tag!("abcd")                   ~
      count_fixed!( u16, le_u16, 4 ) ~
      eof!()                         ,
      || { () }
    )
  }

  #[test]
  fn count_fixed_no_type() {
    const TIMES: usize = 2;
    named!( tag_abc, tag!("abc") );
    named!( counter_2<&[u8], [&[u8]; TIMES], () >, count_fixed!(&[u8], tag_abc, TIMES ) );

    let done = &b"abcabcabcdef"[..];
    let parsed_main = [&b"abc"[..], &b"abc"[..]];
    let rest = &b"abcdef"[..];
    let incomplete_1 = &b"ab"[..];
    let incomplete_2 = &b"abcab"[..];
    let error = &b"xxx"[..];
    let error_1 = &b"xxxabcabcdef"[..];
    let error_1_remain = &b"xxxabcabcdef"[..];
    let error_2 = &b"abcxxxabcdef"[..];
    let error_2_remain = &b"abcxxxabcdef"[..];

    assert_eq!(counter_2(done), Done(rest, parsed_main));
    assert_eq!(counter_2(incomplete_1), Incomplete(Needed::Unknown));
    assert_eq!(counter_2(incomplete_2), Incomplete(Needed::Unknown));
    assert_eq!(counter_2(error), Error(error_position!(ErrorKind::Count, error)));
    assert_eq!(counter_2(error_1), Error(error_position!(ErrorKind::Count, error_1_remain)));
    assert_eq!(counter_2(error_2), Error(error_position!(ErrorKind::Count, error_2_remain)));
  }

  #[test]
  fn length_value_test() {
    named!(length_value_1<&[u8], Vec<u16> >, length_value!(be_u8, be_u16));
    named!(length_value_2<&[u8], Vec<u16> >, length_value!(be_u8, be_u16, 2));

    let i1 = vec![0, 5, 6];
    assert_eq!(length_value_1(&i1), IResult::Done(&i1[1..], vec![]));
    assert_eq!(length_value_2(&i1), IResult::Done(&i1[1..], vec![]));

    let i2 = vec![1, 5, 6, 3];
    assert_eq!(length_value_1(&i2), IResult::Done(&i2[3..], vec![1286]));
    assert_eq!(length_value_2(&i2), IResult::Done(&i2[3..], vec![1286]));

    let i3 = vec![2, 5, 6, 3, 4, 5, 7];
    assert_eq!(length_value_1(&i3), IResult::Done(&i3[5..], vec![1286, 772]));
    assert_eq!(length_value_2(&i3), IResult::Done(&i3[5..], vec![1286, 772]));

    let i4 = vec![2, 5, 6, 3];
    assert_eq!(length_value_1(&i4), IResult::Incomplete(Needed::Size(5)));
    assert_eq!(length_value_2(&i4), IResult::Incomplete(Needed::Size(5)));

    let i5 = vec![3, 5, 6, 3, 4, 5];
    assert_eq!(length_value_1(&i5), IResult::Incomplete(Needed::Size(7)));
    assert_eq!(length_value_2(&i5), IResult::Incomplete(Needed::Size(7)));
  }

  #[test]
  fn fold_many0() {
    fn fold_into_vec<T>(mut acc: Vec<T>, item: T) -> Vec<T> {
      acc.push(item);
      acc
    };
    named!( tag_abcd, tag!("abcd") );
    named!( tag_empty, tag!("") );
    named!( multi<&[u8],Vec<&[u8]> >, fold_many0!(tag_abcd, Vec::new(), fold_into_vec) );
    named!( multi_empty<&[u8],Vec<&[u8]> >, fold_many0!(tag_empty, Vec::new(), fold_into_vec) );

    assert_eq!(multi(&b"abcdef"[..]), Done(&b"ef"[..], vec![&b"abcd"[..]]));
    assert_eq!(multi(&b"abcdabcdefgh"[..]), Done(&b"efgh"[..], vec![&b"abcd"[..], &b"abcd"[..]]));
    assert_eq!(multi(&b"azerty"[..]), Done(&b"azerty"[..], Vec::new()));
    assert_eq!(multi(&b"abcdab"[..]), Incomplete(Needed::Size(8)));
    assert_eq!(multi(&b"abcd"[..]), Done(&b""[..], vec![&b"abcd"[..]]));
    assert_eq!(multi(&b""[..]), Done(&b""[..], Vec::new()));
    assert_eq!(multi_empty(&b"abcdef"[..]), Error(error_position!(ErrorKind::Many0, &b"abcdef"[..])));
  }

  #[test]
  fn fold_many1() {
    fn fold_into_vec<T>(mut acc: Vec<T>, item: T) -> Vec<T> {
      acc.push(item);
      acc
    };
    named!(multi<&[u8],Vec<&[u8]> >, fold_many1!(tag!("abcd"), Vec::new(), fold_into_vec));

    let a = &b"abcdef"[..];
    let b = &b"abcdabcdefgh"[..];
    let c = &b"azerty"[..];
    let d = &b"abcdab"[..];

    let res1 = vec![&b"abcd"[..]];
    assert_eq!(multi(a), Done(&b"ef"[..], res1));
    let res2 = vec![&b"abcd"[..], &b"abcd"[..]];
    assert_eq!(multi(b), Done(&b"efgh"[..], res2));
    assert_eq!(multi(c), Error(error_position!(ErrorKind::Many1,c)));
    assert_eq!(multi(d), Incomplete(Needed::Size(8)));
  }

  #[test]
  fn fold_many_m_n() {
    fn fold_into_vec<T>(mut acc: Vec<T>, item: T) -> Vec<T> {
      acc.push(item);
      acc
    };
    named!(multi<&[u8],Vec<&[u8]> >, fold_many_m_n!(2, 4, tag!("Abcd"), Vec::new(), fold_into_vec));

    let a = &b"Abcdef"[..];
    let b = &b"AbcdAbcdefgh"[..];
    let c = &b"AbcdAbcdAbcdAbcdefgh"[..];
    let d = &b"AbcdAbcdAbcdAbcdAbcdefgh"[..];
    let e = &b"AbcdAb"[..];

    assert_eq!(multi(a), Error(error_position!(ErrorKind::ManyMN,a)));
    let res1 = vec![&b"Abcd"[..], &b"Abcd"[..]];
    assert_eq!(multi(b), Done(&b"efgh"[..], res1));
    let res2 = vec![&b"Abcd"[..], &b"Abcd"[..], &b"Abcd"[..], &b"Abcd"[..]];
    assert_eq!(multi(c), Done(&b"efgh"[..], res2));
    let res3 = vec![&b"Abcd"[..], &b"Abcd"[..], &b"Abcd"[..], &b"Abcd"[..]];
    assert_eq!(multi(d), Done(&b"Abcdefgh"[..], res3));
    assert_eq!(multi(e), Incomplete(Needed::Size(8)));
  }

}
