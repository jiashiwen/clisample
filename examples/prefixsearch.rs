// #[derive(Copy)]
#[derive(Debug)]
pub struct PrefixIterm {
    pub level: usize,
    pub iterm: String,
    pub children: Vec<String>,
}

pub struct PrefixCompleter {
    prefix_iterms: Vec<PrefixIterm>,
}

impl PrefixCompleter {
    pub fn new() -> Self {
        Self {
            prefix_iterms: vec![]
        }
    }
    pub fn add(&mut self, pi: PrefixIterm) {
        self.prefix_iterms.push(pi);
    }

    pub fn getLevelIterms(&self, level: usize) -> Vec<Box<&PrefixIterm>> {
        let mut itermvec = vec![];

        for iterm in &self.prefix_iterms {
            if iterm.level == level {
                itermvec.push(Box::new(iterm));
            }
        }

        return itermvec;
    }

    pub fn get_level_posible(&self, level: usize, prefix: &str) -> Vec<String> {
        let mut strvec: Vec<String> = vec![];
        let leveliterms = self.getLevelIterms(level);
        for iterm in leveliterms {
            if iterm.iterm.starts_with(prefix) {
                strvec.push(iterm.iterm.clone());
            }
        }
        return strvec;
    }
    pub fn get_level_iterm_posible(&self, level: usize, iterm: String, prefix: &str) -> Vec<String> {
        let mut strvec = vec![];
        // let mut corectiterm: Box<&PrefixIterm>;
        let leveliterms = self.getLevelIterms(level);
        for literm in leveliterms {
            if literm.iterm.eq(&iterm) {
                // corectiterm = literm.clone();
                for str in &literm.children {
                    if str.starts_with(prefix) {
                        strvec.push(str.clone());
                    }
                }
            }
        }

        // for str in corectiterm.children {
        //     if str.starts_with(prefix) {
        //         strvec.push(str.clone());
        //     }
        // }


        return strvec;
    }
}


fn main() {
    let mut pc = PrefixCompleter::new();
    pc.add(PrefixIterm {
        level: 0,
        iterm: "abc".to_string(),
        children: vec!["abc".to_string(), "defg".to_string(), "dfds".to_string(), "dds".to_string()],
    });
    pc.add(PrefixIterm {
        level: 0,
        iterm: "bcd".to_string(),
        children: vec!["abc".to_string(), "defg".to_string(), "dfds".to_string(), "dds".to_string()],
    });
    pc.add(PrefixIterm {
        level: 0,
        iterm: "bcsds".to_string(),
        children: vec!["abc".to_string(), "defg".to_string(), "dfds".to_string(), "dds".to_string()],
    });
    pc.add(PrefixIterm {
        level: 0,
        iterm: "bdsggg".to_string(),
        children: vec!["abc".to_string(), "def".to_string(), "dfs".to_string(), "deds".to_string()],
    });
    pc.add(PrefixIterm {
        level: 0,
        iterm: "bcsggg".to_string(),
        children: vec!["abc".to_string(), "defg".to_string(), "dfds".to_string(), "dds".to_string()],
    });
    pc.add(PrefixIterm {
        level: 1,
        iterm: "bcddsaf".to_string(),
        children: vec!["abc".to_string(), "defg".to_string(), "dfds".to_string(), "dds".to_string()],
    });
    let levelzero = pc.getLevelIterms(1);
    let searchresult = pc.get_level_posible(0, "bcs");
    let levelitermsearch = pc.get_level_iterm_posible(0, "bdsggg".to_string(), "de");
    println!("{:?}", levelzero);
    println!("{:?}", searchresult);
    println!("{:?}", levelitermsearch);
}