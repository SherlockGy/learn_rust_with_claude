struct MiniVec<T> {
    data: Vec<T>,
}

impl<T> MiniVec<T> {
    fn new() -> MiniVec<T> {
        MiniVec { data: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.data.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn first(&self) -> Option<&T> {
        self.data.first()
    }

    fn last(&self) -> Option<&T> {
        self.data.last()
    }
}

impl<T: Clone> MiniVec<T> {
    fn to_vec(&self) -> Vec<T> {
        self.data.clone()
    }

    fn first_clone(&self) -> Option<T> {
        self.data.first().cloned()
    }
}

impl<T: PartialEq> MiniVec<T> {
    fn contains(&self, item: &T) -> bool {
        self.data.contains(item)
    }

    fn position(&self, item: &T) -> Option<usize> {
        self.data.iter().position(|x| x == item)
    }
}

impl<T: std::fmt::Debug> MiniVec<T> {
    fn debug_print(&self) {
        println!("MiniVec({:?})", self.data);
    }
}

fn main() {
    println!("=== MiniVec 演示 ===\n");

    // 基本操作
    let mut vec: MiniVec<i32> = MiniVec::new();
    println!("创建空 MiniVec");
    println!("长度: {}", vec.len());
    println!("为空: {}\n", vec.is_empty());

    // push 操作
    println!("添加元素: 1, 2, 3");
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.debug_print();
    println!();

    // 访问元素
    println!("第一个元素: {:?}", vec.first());
    println!("最后一个元素: {:?}", vec.last());
    println!("索引 1 处: {:?}\n", vec.get(1));

    // 查找
    println!("包含 2: {}", vec.contains(&2));
    println!("包含 5: {}", vec.contains(&5));
    println!("2 的位置: {:?}\n", vec.position(&2));

    // pop 操作
    println!("弹出: {:?}", vec.pop());
    vec.debug_print();
    println!("新长度: {}\n", vec.len());

    // 字符串类型
    println!("=== 字符串类型 ===\n");
    let mut string_vec: MiniVec<String> = MiniVec::new();
    string_vec.push(String::from("Hello"));
    string_vec.push(String::from("Rust"));
    string_vec.push(String::from("World"));
    string_vec.debug_print();

    println!("包含 \"Rust\": {}", string_vec.contains(&String::from("Rust")));
    println!("第一个克隆: {:?}", string_vec.first_clone());

    println!("\n=== 演示完成 ===");
}
