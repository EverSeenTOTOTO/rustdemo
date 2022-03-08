pub fn print_addr<T>(name: &str, reference: &T) {
    println!("addr of {} = {:#?}", name, reference as *const _);
}

pub fn test_size_of() {
    use std::mem::size_of_val;

    let v1 = vec![1, 2, 3];
    let v2 = vec![4, 5, 6];

    {
        let simple = v1.iter();
        println!("size of simple = {} bytes", size_of_val(&simple));
    }

    {
        let chained = v1.iter().chain(v2.iter());
        println!("size of chained = {} bytes", size_of_val(&chained));
    }

    {
        let vv = vec![v1, v2];
        let flattened = vv.iter().flatten();
        println!("size of flattened = {} bytes", size_of_val(&flattened))
    }
}

pub fn test_pointer_size() {
    use std::mem::size_of_val;

    let v1 = vec![1, 2, 3];
    let v2 = vec![4, 5, 6];
    {
        let simple = Box::new(v1.iter());
        println!("size of boxed simple = {} bytes", size_of_val(&simple));
    }

    {
        let chained = Box::new(v1.iter().chain(v2.iter()));
        println!("size of boxed chained = {} bytes", size_of_val(&chained));
    }

    {
        let vv = vec![v1, v2];
        let flattened = Box::new(vv.iter().flatten());
        println!(
            "size of boxed flattened = {} bytes",
            size_of_val(&flattened)
        );
    }
}

pub fn test_addr() {
    let v1 = vec![1, 2, 3];
    let v2 = vec![4, 5, 6];

    print_addr("v1", &v1);
    print_addr("v2", &v2);

    {
        let simple = Box::new(v1.iter());
        println!("~~ simple ~~");
        print_addr("box     ", &simple);
        print_addr("contents", &*simple);
    }
}
