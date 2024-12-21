fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(14, 15), 1);

    assert_eq!(gcd(2 * 3 * 5 * 11 * 17,
                   3 * 7 * 11 * 13 * 19),
               3 * 11);
}

use std::str::FromStr;
use std::env;

fn main() {
    let mut numbers = Vec::new();

    for arg in env::args().skip(1) {
        numbers.push(u64::from_str(&arg)
                     .expect("error parsing argument"));
    }

    if numbers.len() == 0 {
        eprintln!("Usage: gcd NUMBER ...");
        std::process::exit(1);
    }

    let mut d = numbers[0];
    for m in &numbers[1..] {
        d = gcd(d, *m);
    }

    println!("The greatest common divisor of {:?} is {}",
             numbers, d);
}

fn grep<R>(target: &str, reader: R) -> io::Result<()>
    where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(target) {
            println!("{}", line);
        }
    }
    Ok(())
}

struct Request {
    method: String,
    url: String,
    headers: HashMap<String, String>,
    body: Vec<u8>
}

struct Response {
    code: u32,
    headers: HashMap<String, String>,
    body: Vec<u8>
}

type BoxedCallback = Box<dyn Fn(&Request) -> Response>;

struct BasicRouter {
    routes: HashMap<String, BoxedCallback>
}

impl BasicRouter {
    // Create an empty router.
    fn new() -> BasicRouter {
        BasicRouter { routes: HashMap::new() }
    }

    // Add a route to the router.
    fn add_route<C>(&mut self, url: &str, callback: C)
        where C: Fn(&Request) -> Response + 'static
    {
        self.routes.insert(url.to_string(), Box::new(callback));
    }
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    let parker = Parker::new();
    let unparker = parker.unparker().clone();
    let waker = waker_fn(move || unparker.unpark());
    let mut context = Context::from_waker(&waker);

    pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => parker.park(),
        }
    }
}

macro_rules! define_complex {
    () => {
        #[derive(Clone, Copy, Debug)]
        struct Complex<T> {
            /// Real portion of the complex number
            re: T,

            /// Imaginary portion of the complex number
            im: T,
        }
    };
}

mod very_generic {
    define_complex!();

    use std::ops::Add;

    impl<L, R> Add<Complex<R>> for Complex<L>
    where
        L: Add<R>,
    {
        type Output = Complex<L::Output>;
        fn add(self, rhs: Complex<R>) -> Self::Output {
            Complex {
                re: self.re + rhs.re,
                im: self.im + rhs.im,
            }
        }
    }
}

mod impl_compound {
    define_complex!();

    use std::ops::AddAssign;

    impl<T> AddAssign for Complex<T>
    where
        T: AddAssign<T>,
    {
        fn add_assign(&mut self, rhs: Complex<T>) {
            self.re += rhs.re;
            self.im += rhs.im;
        }
    }
}

mod derive_partialeq {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Complex<T> {
        re: T,
        im: T,
    }
}

mod derive_everything {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Complex<T> {
        /// Real portion of the complex number
        re: T,

        /// Imaginary portion of the complex number
        im: T,
    }
}

fn copy_dir_to(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.is_dir() {
        fs::create_dir(dst)?;
    }

    for entry_result in src.read_dir()? {
        let entry = entry_result?;
        let file_type = entry.file_type()?;
        copy_to(&entry.path(), &file_type, &dst.join(entry.file_name()))?;
    }

    Ok(())
}

fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(src: P, _dst: Q) -> std::io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other,
                       format!("can't copy symbolic link: {}",
                               src.as_ref().display())))
}

fn copy_to(src: &Path, src_type: &fs::FileType, dst: &Path) -> io::Result<()> {
    if src_type.is_file() {
        fs::copy(src, dst)?;
    } else if src_type.is_dir() {
        copy_dir_to(src, dst)?;
    } else if src_type.is_symlink() {
        let target = src.read_link()?;
        symlink(target, dst)?;
    } else {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  format!("don't know how to copy: {}",
                                          src.display())));
    }
    Ok(())
}

fn copy_into<P, Q>(source: P, destination: Q) -> io::Result<()>
    where P: AsRef<Path>,
          Q: AsRef<Path>
{
    let src = source.as_ref();
    let dst = destination.as_ref();

    match src.file_name() {
        None => {
            return Err(io::Error::new(io::ErrorKind::Other,
                                      format!("can't copy nameless directory: {}",
                                              src.display())));
        }
        Some(src_name) => {
            let md = src.metadata()?;
            copy_to(src, &md.file_type(), &dst.join(src_name))?;
        }
    }
    Ok(())
}

fn complex() {
    #[derive(Copy, Clone, Debug)]
    struct Complex { re: f64, im: f64 }

    let third = Complex { re: -0.5, im: f64::sqrt(0.75) };
    println!("{:?}", third);

    use std::fmt;

    impl fmt::Display for Complex {
        fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
            let im_sign = if self.im < 0.0 { '-' } else { '+' };
            write!(dest, "{} {} {}i", self.re, im_sign, f64::abs(self.im))
        }
    }

    let one_twenty = Complex { re: -0.5, im: 0.866 };
    assert_eq!(format!("{}", one_twenty),
               "-0.5 + 0.866i");

    let two_forty = Complex { re: -0.5, im: -0.866 };
    assert_eq!(format!("{}", two_forty),
               "-0.5 - 0.866i");
}

mod non_generic_add {
    define_complex!();

    use std::ops::Add;

    impl Add for Complex<i32> {
        type Output = Complex<i32>;
        fn add(self, rhs: Self) -> Self {
            Complex {
                re: self.re + rhs.re,
                im: self.im + rhs.im,
            }
        }
    }
}

mod non_generic_add {
    define_complex!();

    use std::ops::Add;

    impl Add for Complex<i32> {
        type Output = Complex<i32>;
        fn add(self, rhs: Self) -> Self {
            Complex {
                re: self.re + rhs.re,
                im: self.im + rhs.im,
            }
        }
    }
}

enum BinaryTree<T> {
    Empty,
    NonEmpty(Box<TreeNode<T>>),
}

struct TreeNode<T> {
    element: T,
    left: BinaryTree<T>,
    right: BinaryTree<T>,
}

struct TreeIter<'a, T> {
    // A stack of references to tree nodes. Since we use `Vec`'s
    // `push` and `pop` methods, the top of the stack is the end of the
    // vector.
    //
    // The node the iterator will visit next is at the top of the stack,
    // with those ancestors still unvisited below it. If the stack is empty,
    // the iteration is over.
    unvisited: Vec<&'a TreeNode<T>>
}

#[test]
fn binary_tree_size() {
    use std::mem::size_of;

    let word = size_of::<usize>();
    assert_eq!(size_of::<BinaryTree<String>>(), word);
    type Triple = (&'static str, BinaryTree<&'static str>, BinaryTree<&'static str>);
    assert_eq!(size_of::<Triple>(), 4 * word);
}

async fn post_gcd(form: web::Form<GcdParameters>) -> HttpResponse {
    if form.n == 0 || form.m == 0 {
        return HttpResponse::BadRequest()
            .content_type("text/html")
            .body("Computing the GCD with zero is boring.");
    }

    let response =
        format!("The greatest common divisor of the numbers {} and {} \
                 is <b>{}</b>\n",
                form.n, form.m, gcd(form.n, form.m));

    HttpResponse::Ok()
        .content_type("text/html")
        .body(response)
}

use actix_web::{web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() {
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_index))
            .route("/gcd", web::post().to(post_gcd))
    });

    println!("Serving on http://localhost:3000...");
    server
        .bind("127.0.0.1:3000").expect("error binding server to address")
        .run()
        .await
        .expect("error running server");
}