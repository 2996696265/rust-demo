use bracket_lib::prelude::*;
use std::collections::VecDeque;
use std::time::{Instant, Duration};

#[derive(Debug, PartialEq, Clone, Copy)]
enum GameMode {
    MainMenu,
    Playing,
    End,
    Records,
    DifficultySelect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        };
        write!(f, "{s}")
    }
}

// 定义游戏记录结构体
#[derive(Debug, Clone, Copy)]
struct GameRecord {
    score: i32,
    duration: f32, // 游戏时长(秒)
    difficulty: Difficulty, // 新增字段：记录当局难度
}


const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;//每帧持续时间，单位毫秒


struct Player{
    x: i32, //
    y: i32,
    health: f32,
}

impl Player{
    fn new(x: i32, y: i32)->Self{
        Player{
            x:0,
            y:0,
            health: 0.0 ,
        }
    }
    fn render(&mut self, ctx: &mut BTerm){
        ctx.set(0,self.y, YELLOW, BLACK, to_cp437('@'));//渲染
    }

    fn move_down(&mut self,difficulty: &Difficulty){
        //if self.health < match difficulty {
        //    Difficulty::Easy => 3.0,
        //    Difficulty::Normal => 2.0,
        //    Difficulty::Hard => 1.5,
        //} {
        //    self.health += 0.2;
        //}
        //if self.health<2.0{//模拟加速度
        //    self.health+=0.2;
        //}
        

        if self.health < match difficulty {
            Difficulty::Easy => 3.0,
            Difficulty::Normal => 2.0,
            Difficulty::Hard => 1.5,
        } {
            self.health += 0.2;
        }
        self.y += self.health as i32;

        let dx = match difficulty {
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard => 2,
        };

        self.x += dx;//每个tick移动一格

        if self.y<0{
            self.y=0;
        }
    }
    fn flap(&mut self){
        self.health = -2.0;//按一下空格，改变速度（方向）     
        
    }
}

struct Structtate{ 
    player: Player,
    frame_time: f32,
    mode: GameMode,
    object:Object,
    score:i32,
    start_time: f32, // 游戏开始时间
    game_records: VecDeque<GameRecord>, // 存储游戏记录
    record_page_frames: i32,  // 新增字段，记录在记录页面停留的帧数
    difficulty: Difficulty,

}


impl Structtate{
    fn new()->Self{
        Structtate{//返回初始状态，返回菜单
            player: Player::new(5,25),
            frame_time:0.0,
            mode: GameMode::MainMenu,
            object:Object::new(SCREEN_WIDTH,0,Difficulty::Normal),
            score:0,
            start_time: 0.0,
            game_records: VecDeque::with_capacity(5), // 初始化容量为5的记录队列
            record_page_frames: 0,  // 初始化为0
            difficulty: Difficulty::Normal,
        }
    }
    // 添加游戏记录
    fn add_game_record(&mut self, score: i32, duration: f32) {
        // 辅助函数：将难度映射为数值，便于排序
        fn difficulty_value(difficulty: Difficulty) -> i32 {
            match difficulty {
                Difficulty::Easy => 0,
                Difficulty::Normal => 1,
                Difficulty::Hard => 2,
            }
        }   
        let record = GameRecord { score, duration, difficulty: self.difficulty };
    
        if self.game_records.len() == 5 {
            if let Some(min_index) = self.game_records.iter().enumerate()
                .min_by_key(|(_, r)| (r.score, difficulty_value(r.difficulty))) // 注意：分数最小 + 难度最低
                .map(|(i, _)| i)
            {
                self.game_records.remove(min_index);
            }
        }
    
        self.game_records.push_back(record);
        self.game_records.make_contiguous().sort_by(|a, b| {
            // 排序规则：先按 score 降序，再按 difficulty 降序
            b.score.cmp(&a.score)
                .then_with(|| difficulty_value(b.difficulty).cmp(&difficulty_value(a.difficulty)))
        });
    }
    
    
    
    // 显示游戏记录
    fn show_game_records(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(2, "=== HIGH SCORES ===");
        
        for (i, record) in self.game_records.iter().enumerate() {
            let minutes = (record.duration / 60.0) as i32;
            let seconds = (record.duration % 60.0) as i32;
            ctx.print(
                2, 
                4 + i as i32, 
                //&format!("{}. Score: {}, Time: {}:{:02}", i+1, record.score, minutes, seconds)
                &format!(
                    "{}. [{}] Score: {}, Time: {}:{:02}",
                    i + 1,
                    record.difficulty,
                    record.score,
                    minutes,
                    seconds
                )
            );
        }
        
        ctx.print_centered(12, "[F] Play Again    [Esc] Main Menu");
        // 至少显示1帧后才开始检测按键
        //self.record_page_frames += 1;
        //if self.record_page_frames > 1000 {
        //    if let Some(_) = ctx.key_pressed()  {
        //        self.mode = GameMode::MainMenu;
        //        self.record_page_frames = 0;  // 重置帧计数
        //    }
        //}
        
        // 检测按键
        self.record_page_frames += 1;
        if self.record_page_frames > 10 {
            if let Some(key) = ctx.key {
                match key {
                    VirtualKeyCode::Escape => {
                        self.mode = GameMode::MainMenu;
                        self.record_page_frames = 0;
                    }
                    VirtualKeyCode::P | VirtualKeyCode::F => {
                        self.restart(); // 调用 restart()，重启游戏
                    }
                    _ => {
                        self.mode = GameMode::MainMenu; // 也可以做成“任意键返回”
                        self.record_page_frames = 0;
                    }
                }
            }
        }
    }

    fn difficulty_select(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Select Difficulty:");
        ctx.print_centered(7, "[1] Easy");
        ctx.print_centered(8, "[2] Normal");
        ctx.print_centered(9, "[3] Hard");
        ctx.print_centered(11, "Press Esc to return");
    
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Key1 => {
                    self.difficulty = Difficulty::Easy;
                    self.restart(); // 进入游戏
                }
                VirtualKeyCode::Key2 => {
                    self.difficulty = Difficulty::Normal;
                    self.restart();
                }
                VirtualKeyCode::Key3 => {
                    self.difficulty = Difficulty::Hard;
                    self.restart();
                }
                VirtualKeyCode::Escape => {
                    self.mode = GameMode::MainMenu;
                }
                _ => {}
            }
        }
    }
    


    fn playing(& mut self, ctx: &mut BTerm){
        ctx.cls_bg(NAVY);//清屏
        
        self.frame_time+=ctx.frame_time_ms;//累加帧时间
        if self.frame_time>FRAME_DURATION{//如果帧时间大于每帧持续时间
            self.frame_time=0.0;//减去每帧持续时间
            self.player.move_down(&self.difficulty);//移动
        }
        if let Some(VirtualKeyCode::Space)=ctx.key{//设置输入
            self.player.flap();
        }
        self.player.render(ctx);//渲染
        ctx.print(0,0,"Hello,Press Sapce to Flap!");
        ctx.print(0, 1, &format!("Difficulty: {}", self.difficulty));
        ctx.print(0,2,&format!("Score:{}",self.score));
        self.object.render(ctx,self.player.x);
        if self.player.x>self.object.x{
            self.score+=1;
            //self.object=Object::new(self.player.x + SCREEN_WIDTH,self.score);
            self.object = Object::new(self.player.x + SCREEN_WIDTH, self.score, self.difficulty);
        }

        if self.player.y>SCREEN_HEIGHT ||self.object.hit(&self.player){//如果玩家超出屏幕
            // 计算游戏时长(秒)
            let duration = (self.frame_time / 1000.0) + (ctx.frame_time_ms / 1000.0); // 近似计算总时长
            self.add_game_record(self.score, duration);
            self.mode=GameMode::End;//模式转为结束模式
        }
        
        //self.mode=GameMode::End;//模式转为结束模式
    }
    fn restart(& mut self){//重置
        self.player=Player::new(5,25);
        self.frame_time=0.0;
        self.mode=GameMode::Playing;//模式转为游戏进行中模式
        self.object=Object::new(SCREEN_WIDTH,0, self.difficulty);
        self.score=0;
        self.start_time = 0.0; // 重置开始时间
    }
    fn main_menu(&mut self, ctx: &mut BTerm){
        ctx.cls();//清屏
        ctx.print_centered(5,"Welcome to the Hello Game!");//输出菜单
        ctx.print_centered(8,"Press F to start the game!");
        ctx.print_centered(9,"Press Q to quit the game!");
        ctx.print_centered(10,"Press T to view high scores!"); // 添加查看记录的提示
        ctx.print_centered(11, "Press D to select difficulty!");

        if let Some(key)=ctx.key{//设置输入
            match key{
                VirtualKeyCode::F=> self.restart(),
                VirtualKeyCode::Q=> ctx.quitting=true,
                //VirtualKeyCode::T=> self.show_game_records(ctx), // 按T查看记录
                VirtualKeyCode::T => {
                    self.record_page_frames = 0;
                    self.mode = GameMode::Records;
                }
                VirtualKeyCode::D => self.mode = GameMode::DifficultySelect,
                _ => {}
            }
        }
    }
    fn dead(& mut self, ctx: &mut BTerm){//游戏结束
        ctx.cls();//清屏
        ctx.print_centered(5,"You failed!");
        ctx.print_centered(6,&format!("Your score is: {}",self.score));
        ctx.print_centered(8,"Press F to play again!");
        ctx.print_centered(9,"Press Q to quit the game!");
        ctx.print_centered(10,"Press T to view high scores!"); // 添加查看记录的提示
        ctx.print_centered(11, "Press D to select difficulty!");
        if let Some(key)=ctx.key{
            match key{
                VirtualKeyCode::F=> self.restart(),
                VirtualKeyCode::Q=> ctx.quitting=true,
                //VirtualKeyCode::T=> self.show_game_records(ctx), // 按T查看记录
                VirtualKeyCode::T => {
                    self.record_page_frames = 0;
                    self.mode = GameMode::Records;
                }
                VirtualKeyCode::D => self.mode = GameMode::DifficultySelect,
                _ => {}
            }
        }


    }
}



impl GameState for Structtate {//impl是一个关键字，用于实现一个trait，GameMode是一个trait，用于定义游戏的逻辑
    fn tick(&mut self, ctx: &mut BTerm) {//ctx是一个上下文
        //ctx.cls();//清屏
        //ctx.print(1,1,"Hello!");
        match self.mode{//match是一个关键字，用于匹配一个值，判断游戏当前状态，来执行对应方法
            GameMode::MainMenu=> self.main_menu(ctx),//self.main_menu(ctx)是一个函数，用于处理主菜单的逻辑
            GameMode::End=>self.dead(ctx),//self.dead(ctx)是一个函数，用于处理游戏结束的逻辑
            GameMode::Playing=>self.playing(ctx),//self.playing(ctx)是一个函数，用于处理游戏进行中的逻辑
            GameMode::Records => self.show_game_records(ctx), // 添加查看记录的逻辑
            GameMode::DifficultySelect => self.difficulty_select(ctx),
        }
    }
}

struct Object{
    x: i32,  //
    y: i32,
    size:i32,
}

impl Object{
    fn new(x: i32, score:i32,difficulty: Difficulty) -> Self {
        //let base_size = match score {
        //    s if s < 5 => 10,
        //    s if s < 10 => 8,
        //    _ => 6,
        //};
        
        //let difficulty_modifier = match difficulty {
        //    Difficulty::Easy => 2,
        //    Difficulty::Normal => 0,
        //    Difficulty::Hard => -2,
        //};
        // 根据难度设定最小间隙大小
        
        
        //let max_size = (base_size + difficulty_modifier).max(2);
        let mut random=RandomNumberGenerator::new();
        // 确保 20 - score >= 2，否则设置最小尺寸
        let min_size = 2;
        let min_size = match difficulty {
            Difficulty::Easy => 10,
            Difficulty::Normal => 6,
            Difficulty::Hard => 2,
        };
        let max_size = (20 - score).max(min_size);  // 确保 max_size 不小于 min_size
        Object {
            x,
            y:random.range(10, SCREEN_HEIGHT-10),
            size: random.range(min_size, max_size), 
        }
        
    }
    
    fn render(&mut self, ctx: &mut BTerm,playerx:i32){
        let screenx=self.x -playerx;
        let halfsize=self.size/2;
        //for y in 0..self.y-halfsize{
        //    ctx.set(screanx,y,RED,BLACK,to_cp437('H'));
        //}
        //for y in self.y + halfsize..SCREEN_HEIGHT{
        //     ctx.set(screanx, y, RED, BLACK, to_cp437('H'));
        //}
        // 创建一个循环迭代器来循环显示"HELLO"
        let hello_chars = ['H', 'E', 'L', 'L', 'O'].iter().cycle();
    
        // 上半部分障碍物
        for y in 0..self.y - halfsize {
            // 计算当前应该显示哪个字符(循环)
            let char_index = ((y / 2) % 5) as usize;  // 每2行循环一次，5个字符循环
            let current_char = ['H', 'E', 'L', 'L', 'O'][char_index];
            ctx.set(screenx, y, RED, BLACK, to_cp437(current_char));
        }
        
        // 下半部分障碍物
        for y in self.y + halfsize..SCREEN_HEIGHT {
            // 计算当前应该显示哪个字符(循环)
            let char_index = ((y / 2) % 5) as usize;  // 每2行循环一次，5个字符循环
            let current_char = ['H', 'E', 'L', 'L', 'O'][char_index];
            ctx.set(screenx, y, RED, BLACK, to_cp437(current_char));
        }

       
    }

    fn hit(&self,player:&Player)->bool{//
        let halfsize = self.size/2;
        let does_x_match = player.x == self.x ;
        let is_player_above = player.y < self.y-halfsize;
        let is_player_below = player.y > self.y + halfsize;
        does_x_match && (is_player_above || is_player_below)
    }
}


fn main()->BError {//BError是一个枚举类型
    let context= BTermBuilder::simple80x50()//建立一个80x50的窗口
        .with_title("Hello")
        .build()?;//建立实例//?是一个宏，用于处理错误，出错则返回错误BError 
    main_loop(context, Structtate::new())//游戏主循环
    
    //println!("Hello, world!");
}


pub trait MenuCtx {
    fn get_key(&self) -> Option<VirtualKeyCode>;
    fn set_quitting(&mut self, b:bool);
    fn cls(&mut self);
    fn print_centered(&mut self, _y:i32, _str:&str);
}
#[derive(Default)]
pub struct FakeBTerm {
    pub key: Option<VirtualKeyCode>,
    pub quitting: bool,
}
impl MenuCtx for FakeBTerm {
    fn get_key(&self) -> Option<VirtualKeyCode> { self.key }
    fn set_quitting(&mut self, b:bool) { self.quitting = b; }
    fn cls(&mut self) {}
    fn print_centered(&mut self, _y:i32, _s:&str) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{MenuCtx, FakeBTerm, Structtate, GameMode, Difficulty};
    use bracket_lib::prelude::VirtualKeyCode;

    // 测试主菜单 F 键开始游戏
    #[test]
    fn tc_01_mainmenu_f_starts_game() {
        let mut game = Structtate::new();
        let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::F), quitting: false };
        // 用 trait 替换 main_menu 的 ctx
        Structtate::main_menu_trait(&mut game, &mut ctx);
        assert_eq!(game.mode, GameMode::Playing);
        assert_eq!(game.difficulty, Difficulty::Normal);
    }

    // 测试主菜单 Q 键退出游戏
    #[test]
    fn tc_02_mainmenu_q_quit_game() {
        let mut game = Structtate::new();
        let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::Q), quitting: false };
        Structtate::main_menu_trait(&mut game, &mut ctx);
        assert!(ctx.quitting);
    }

    // 测试主菜单 T 键进入记录
    #[test]
    fn tc_03_mainmenu_t_to_records() {
        let mut game = Structtate::new();
        let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::T), quitting: false };
        Structtate::main_menu_trait(&mut game, &mut ctx);
        assert_eq!(game.mode, GameMode::Records);
    }

    // 测试主菜单 D 键进难度选择
    #[test]
    fn tc_04_mainmenu_d_to_difficulty_select() {
        let mut game = Structtate::new();
        let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::D), quitting: false };
        Structtate::main_menu_trait(&mut game, &mut ctx);
        assert_eq!(game.mode, GameMode::DifficultySelect);
    }
    #[test]
    fn tc_05_mainmenu_irrelevant_key_no_effect() {
    let mut game = Structtate::new();
    let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::A), quitting: false };
    Structtate::main_menu_trait(&mut game, &mut ctx);
    // 初始 MainMenu，其他按键按下后应该仍然在 MainMenu
    assert_eq!(game.mode, GameMode::MainMenu);
    // quitting 不应被触发
    assert!(!ctx.quitting);
    }

    // 难度选择 1=>Easy,2=>Normal,3=>Hard,ESC回菜单
    #[test]
    fn tc_06_diff_1_to_easy() {
        let mut game = Structtate::new();
        game.mode = GameMode::DifficultySelect;
        let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::Key1), quitting: false };
        Structtate::difficulty_select_trait(&mut game, &mut ctx);
        assert_eq!(game.mode, GameMode::Playing);
        assert_eq!(game.difficulty, Difficulty::Easy);
    }
    #[test]
    fn tc_07_diff_2_to_normal() {
        let mut game = Structtate::new();
        game.mode = GameMode::DifficultySelect;
        let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::Key2), quitting: false };
        Structtate::difficulty_select_trait(&mut game, &mut ctx);
        assert_eq!(game.mode, GameMode::Playing);
        assert_eq!(game.difficulty, Difficulty::Normal);
    }
    #[test]
    fn tc_08_diff_3_to_hard() {
        let mut game = Structtate::new();
        game.mode = GameMode::DifficultySelect;
        let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::Key3), quitting: false };
        Structtate::difficulty_select_trait(&mut game, &mut ctx);
        assert_eq!(game.mode, GameMode::Playing);
        assert_eq!(game.difficulty, Difficulty::Hard);
    }
    #[test]
    fn tc_09_diff_esc_to_mainmenu() {
        let mut game = Structtate::new();
        game.mode = GameMode::DifficultySelect;
        let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::Escape), quitting: false };
        Structtate::difficulty_select_trait(&mut game, &mut ctx);
        assert_eq!(game.mode, GameMode::MainMenu);
    }
    #[test]
    fn tc_10_difficulty_select_irrelevant_key_no_effect() {
    let mut game = Structtate::new();
    game.mode = GameMode::DifficultySelect;
    let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::Z), quitting: false };
    Structtate::difficulty_select_trait(&mut game, &mut ctx);
    // 仍留在难度选择界面
    assert_eq!(game.mode, GameMode::DifficultySelect);
    }

    ///  未操作时小球被重力下落
    #[test]
    fn tc_11_player_gravity_move_down() {
        let mut player = Player::new(0, 10);
        let start_y = player.y;
        // 不做任何操作，模拟多帧调用 move_down
        for _ in 0..5 {
            player.move_down(&Difficulty::Normal);
        }
        assert!(player.y > start_y, "小球y坐标应该增加（下落）");
    }

    #[test]
    fn tc_12_player_flap_and_move_up() {
    let mut player = Player::new(0, 20);
    player.y = 20; // 保证测试点高于0
    let y_before = player.y;
    player.flap();
    player.move_down(&Difficulty::Normal);
    let y_after = player.y;
    println!("y_before: {}, y_after: {}", y_before, y_after);
    assert!(y_after < y_before, "flap后第一帧应上升(y变小)");
    }

    #[test]
    fn tc_13_playing_irrelevant_key_no_flap() {
    let mut game = Structtate::new();
    game.mode = GameMode::Playing;
    game.player.y = 10;
    // 模拟一个不相关按键
    let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::A), quitting: false };
    let y_before = game.player.y;
    let health_before = game.player.health;
    if let Some(VirtualKeyCode::Space) = ctx.key {
        game.player.flap();
    }
    // 不处理move_down，不会因为A键触发改变
    assert_eq!(game.player.y, y_before, "无关按键不应改变y");
    assert_eq!(game.player.health, health_before, "无关按键不应改变health");
    }

    /// 难度和分数实时变量可正常读取
    #[test]
    fn tc_14_difficulty_and_score_displayable() {
        let mut game = Structtate::new();
        // 设置不同难度和得分，模拟一次得分增长
        game.difficulty = Difficulty::Hard;
        game.score = 8;

        // 检查变量
        assert_eq!(game.difficulty, Difficulty::Hard);
        assert_eq!(game.score, 8);
        // 游戏页面右上角显示的是这两项，变量正确即可信任UI显示
    }


    #[test]
    fn tc_15_end_screen_ui_restart_on_f() {

    let mut game = Structtate::new();
    game.mode = GameMode::End;
    game.score = 0;
    let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::F), quitting: false };
    Structtate::dead_trait(&mut game, &mut ctx);
    assert_eq!(game.mode, GameMode::Playing);
    }

    #[test]
    fn tc_16_end_screen_ui_quit_on_q() {
    let mut game = Structtate::new();
    game.mode = GameMode::End;
    let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::Q), quitting: false };
    Structtate::dead_trait(&mut game, &mut ctx);
    assert!(ctx.quitting);
    }

    #[test]
    fn tc_17_end_screen_ui_show_records_on_t() {
    let mut game = Structtate::new();
    game.mode = GameMode::End;
    let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::T), quitting: false };
    Structtate::dead_trait(&mut game, &mut ctx);
    assert_eq!(game.mode, GameMode::Records);
    }

    #[test]
    fn tc_18_end_screen_ui_difficulty_on_d() {
    let mut game = Structtate::new();
    game.mode = GameMode::End;
    let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::D), quitting: false };
    Structtate::dead_trait(&mut game, &mut ctx);
    assert_eq!(game.mode, GameMode::DifficultySelect);
    }

    #[test]
    fn tc_19_end_screen_ui_default_on_other_keys() {
    let mut game = Structtate::new();
    game.mode = GameMode::End;
    let mut ctx = FakeBTerm { key: Some(VirtualKeyCode::A), quitting: false };
    Structtate::dead_trait(&mut game, &mut ctx);
    assert_eq!(game.mode, GameMode::End);
    }
}


impl Structtate {
    pub fn main_menu_trait<C: MenuCtx>(me: &mut Self, ctx: &mut C) {
        ctx.cls();
        ctx.print_centered(5,"Welcome to the Hello Game!");
        ctx.print_centered(8,"Press F to start the game!");
        ctx.print_centered(9,"Press Q to quit the game!");
        ctx.print_centered(10,"Press T to view high scores!");
        ctx.print_centered(11, "Press D to select difficulty!");
        if let Some(key) = ctx.get_key() {
            match key {
                VirtualKeyCode::F => me.restart(),
                VirtualKeyCode::Q => ctx.set_quitting(true),
                VirtualKeyCode::T => {
                    me.record_page_frames = 0;
                    me.mode = GameMode::Records;
                }
                VirtualKeyCode::D => me.mode = GameMode::DifficultySelect,
                _ => {}
            }
        }
    }
    pub fn difficulty_select_trait<C: MenuCtx>(me: &mut Self, ctx: &mut C) {
        ctx.cls();
        ctx.print_centered(5, "Select Difficulty:");
        ctx.print_centered(7, "[1] Easy");
        ctx.print_centered(8, "[2] Normal");
        ctx.print_centered(9, "[3] Hard");
        ctx.print_centered(11, "Press Esc to return");
        if let Some(key) = ctx.get_key() {
            match key {
                VirtualKeyCode::Key1 => {
                    me.difficulty = Difficulty::Easy;
                    me.restart();
                }
                VirtualKeyCode::Key2 => {
                    me.difficulty = Difficulty::Normal;
                    me.restart();
                }
                VirtualKeyCode::Key3 => {
                    me.difficulty = Difficulty::Hard;
                    me.restart();
                }
                VirtualKeyCode::Escape => {
                    me.mode = GameMode::MainMenu;
                }
                _ => {}
            }
        }
    }
    
    pub fn dead_trait<C: MenuCtx>(me: &mut Self, ctx: &mut C) {
        ctx.cls();
        ctx.print_centered(5,"You failed!");
        ctx.print_centered(6,&format!("Your score is: {}",me.score));
        ctx.print_centered(8,"Press F to play again!");
        ctx.print_centered(9,"Press Q to quit the game!");
        ctx.print_centered(10,"Press T to view high scores!");
        ctx.print_centered(11, "Press D to select difficulty!");
        if let Some(key) = ctx.get_key() {
            match key {
                VirtualKeyCode::F => me.restart(),
                VirtualKeyCode::Q => ctx.set_quitting(true),
                VirtualKeyCode::T => {
                    me.record_page_frames = 0;
                    me.mode = GameMode::Records;
                }
                VirtualKeyCode::D => me.mode = GameMode::DifficultySelect,
                _ => {} 
            }
        }
    }
}

