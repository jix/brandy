pub mod testing {
    use std::marker::PhantomData;

    use brandy::{Brand, FreshBrand};

    #[repr(transparent)]
    pub struct BrandedVec<T, B: Brand> {
        vec: Vec<T>,
        _brand: PhantomData<B>,
    }

    #[repr(transparent)]
    pub struct BrandedIndex<B: Brand> {
        index: usize,
        _brand: PhantomData<B>,
    }

    impl<B: Brand> Copy for BrandedIndex<B> {}

    impl<B: Brand> Clone for BrandedIndex<B> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<T, B: Brand> std::ops::Deref for BrandedVec<T, B> {
        type Target = Vec<T>;

        fn deref(&self) -> &Self::Target {
            &self.vec
        }
    }

    impl<T, B: Brand> BrandedVec<T, B> {
        pub fn new(fresh: FreshBrand<B>, vec: Vec<T>) -> BrandedVec<T, B> {
            BrandedVec {
                vec,
                _brand: fresh.into(),
            }
        }

        pub fn indices(&self) -> impl Iterator<Item = BrandedIndex<B>> + 'static {
            (0..self.vec.len()).map(|index| BrandedIndex {
                index,
                _brand: PhantomData,
            })
        }
    }

    impl<T, B: Brand> std::ops::Index<BrandedIndex<B>> for BrandedVec<T, B> {
        type Output = T;

        fn index(&self, index: BrandedIndex<B>) -> &Self::Output {
            unsafe { self.vec.get_unchecked(index.index) }
        }
    }
}

pub mod testing2 {
    use super::testing::*;

    use brandy::{BoundBrand, Brand, FreeBrand, FreshBrand, WithFreshBrand};

    fn print_positions<B: Brand>(
        v: &BrandedVec<usize, B>,
        positions: impl IntoIterator<Item = BrandedIndex<B>>,
    ) {
        println!("{:?}", std::any::type_name::<B>());
        for i in positions {
            println!("{:?}", v[i]);
        }
    }

    fn monomorphization_check<B: Brand>(_: &BrandedVec<usize, B>) -> usize {
        monomorphization_check::<B> as usize
    }

    struct VecWithIndices<B: Brand> {
        vec: BrandedVec<usize, B>,
        indices: Vec<BrandedIndex<B>>,
    }

    struct FreeVecWithIndices;
    impl FreeBrand for FreeVecWithIndices {
        type Bind<B: Brand> = VecWithIndices<B>;
    }

    type BoundVecWithIndices = BoundBrand<FreeVecWithIndices>;

    fn create_data<B: Brand>(fresh: FreshBrand<B>) -> [BoundVecWithIndices; 2] {
        let some_vec: Vec<usize> = (0..10).collect();
        let some_other_vec: Vec<usize> = (0..20).collect();

        let (split_fresh, fresh) = fresh.split();

        let some_branded_vec = BrandedVec::new(split_fresh, some_vec);
        let some_other_branded_vec = BrandedVec::new(fresh, some_other_vec);

        let mut some_indices: Vec<_> = some_branded_vec.indices().collect();
        let mut some_other_indices: Vec<_> = some_other_branded_vec.indices().collect();

        some_indices.reverse();
        some_other_indices.pop();

        print_positions(&some_branded_vec, some_indices.iter().copied());
        print_positions(&some_other_branded_vec, some_other_indices.iter().copied());

        println!("{:08x}", monomorphization_check(&some_branded_vec));
        println!("{:08x}", monomorphization_check(&some_other_branded_vec));

        #[cfg(any)]
        {
            // Using mismatched brands is a type error
            print_positions(&some_branded_vec, some_other_indices.iter().copied());
            print_positions(&some_other_branded_vec, some_indices.iter().copied());
        }

        [
            BoundBrand::bind(VecWithIndices {
                vec: some_branded_vec,
                indices: some_indices,
            }),
            BoundBrand::bind(VecWithIndices {
                vec: some_other_branded_vec,
                indices: some_other_indices,
            }),
        ]
    }

    fn use_data<B: Brand>(fresh: FreshBrand<B>, data: [BoundVecWithIndices; 2]) {
        let (split_fresh, fresh) = fresh.split();

        let [some, some_other] = data;

        let VecWithIndices {
            vec: some_branded_vec,
            indices: some_indices,
        } = some.substitute(split_fresh);

        let VecWithIndices {
            vec: some_other_branded_vec,
            indices: some_other_indices,
        } = some_other.substitute(fresh);

        print_positions(&some_branded_vec, some_indices.iter().copied());
        print_positions(&some_other_branded_vec, some_other_indices.iter().copied());

        println!("{:08x}", monomorphization_check(&some_branded_vec));
        println!("{:08x}", monomorphization_check(&some_other_branded_vec));

        #[cfg(any)]
        {
            // Using mismatched brands is a type error
            print_positions(&some_branded_vec, some_other_indices.iter().copied());
            print_positions(&some_other_branded_vec, some_indices.iter().copied());
        }
    }

    struct DoSomething;

    impl WithFreshBrand for DoSomething {
        type Output = ();

        fn with_fresh_brand(fresh: FreshBrand<impl Brand>) -> Self::Output {
            do_something(fresh);
        }
    }

    fn do_something<B: Brand>(fresh: FreshBrand<B>) {
        let (split_fresh, fresh) = fresh.split();
        let data = create_data(split_fresh);
        use_data(fresh, data);
    }

    pub fn test_do_something() {
        DoSomething::run()
    }
}

fn main() {
    testing2::test_do_something();
}
