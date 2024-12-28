use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use log::{error, info};

use leaky_bucket::RateLimiter;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};


//******************* day 12 **************


const EMPTY: char = '⬛';
const COOKIE: char = '🍪';
const MILK: char = '🥛';
const WALL: char = '⬜';

#[derive(Debug)]
enum GameStatus {
    game_on_going(String),
    game_over_with_winner(char),
    game_over_without_winner(String),
}

pub struct Board {
    grid: [[char; 6]; 5],
    grid_without_wall: [[char; 4]; 4],
}

impl Board {
    // 构造函数：初始化 Board 实例
    pub fn new() -> Self {
        Board {
            grid: [
                [WALL, EMPTY, EMPTY, EMPTY, EMPTY, WALL],
                [WALL, EMPTY, EMPTY, EMPTY, EMPTY, WALL],
                [WALL, EMPTY, EMPTY, EMPTY, EMPTY, WALL],
                [WALL, EMPTY, EMPTY, EMPTY, EMPTY, WALL],
                [WALL, WALL, WALL, WALL, WALL, WALL],
            ],
            grid_without_wall: [
                [EMPTY, EMPTY, EMPTY, EMPTY],
                [EMPTY, EMPTY, EMPTY, EMPTY],
                [EMPTY, EMPTY, EMPTY, EMPTY],
                [EMPTY, EMPTY, EMPTY, EMPTY],
            ],
        }
    }

    // 方法：打印 Board 的内容
    fn board_reset(&mut self) {
        self.grid = [
            ['⬜', '⬛', '⬛', '⬛', '⬛', '⬜'],
            ['⬜', '⬛', '⬛', '⬛', '⬛', '⬜'],
            ['⬜', '⬛', '⬛', '⬛', '⬛', '⬜'],
            ['⬜', '⬛', '⬛', '⬛', '⬛', '⬜'],
            ['⬜', '⬜', '⬜', '⬜', '⬜', '⬜'],
        ];

        //self.grid_without_wall = self.get_grid_without_wall();
        self.grid_without_wall = [
            ['⬛', '⬛', '⬛', '⬛'],
            ['⬛', '⬛', '⬛', '⬛'],
            ['⬛', '⬛', '⬛', '⬛'],
            ['⬛', '⬛', '⬛', '⬛'],
        ];
    }

    // 方法：打印 Board 的内容
    fn print_board(&self) {
        for row in &self.grid {
            println!("{:?}", row);
        }
    }

    // convert Board.grid to string
    fn board_to_string(&self) -> String {
        self.grid
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
            + "\n"
    }

    // convert Board.grid_without_wall to string
    fn board_without_wall_to_string(&self) -> String {
        self.grid_without_wall
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
            + "\n"
    }

    fn get_grid_without_wall(&self) -> [[char; 4]; 4] {
        // 创建新的二维数组，仅保留前 4 行的第 2~5 列
        let mut grid_without_wall = [[' '; 4]; 4]; // 初始化一个空的二维数组

        for (i, row) in self.grid.iter().take(4).enumerate() {
            grid_without_wall[i].copy_from_slice(&row[1..5]); // 拷贝第 2~5 列的数据
        }

        grid_without_wall
    }

    fn update_grid(&mut self, row_number: usize, column_number: usize, tile: char) {
        self.grid[row_number][column_number] = tile;
        info!(
            "grid [{}][{}] is updated with {}",
            row_number, column_number, tile
        );
        self.grid_without_wall = self.get_grid_without_wall();
        info!("grid_without_wall updated. ");
    }

    // check if the column is full or not
    // if the colum is full, check if all the element is same.
    fn check_column_full(&self, column_number: usize) -> (bool, usize, bool) {
        info!("checking: {}", column_number);
        let column_to_check: Vec<char> = self
            .grid_without_wall
            .iter()
            .map(|row| row[column_number])
            .collect();
        let mut capacity = 0;
        let mut is_full = false;
        let mut is_same_element = false;
        info!(
            "grid_without_wall is \n{}",
            self.board_without_wall_to_string()
        );
        info!("column_to_check is \n{:?}", column_to_check);

        for i in 0..4 {
            if column_to_check[i] == '⬛' {
                capacity += 1;
            }
        }
        info!("Capacity is  {}", capacity);

        if capacity == 0 {
            is_full = true;

            info!("column_to_check is:  {:?}", column_to_check);

            if column_to_check.iter().all(|&x| x == column_to_check[0]) {
                is_same_element = true;
                info!(
                    "In this collumn, all elements are same: {}",
                    column_to_check[0]
                );
            } else {
                info!("In this collumn, NOT all elements are same.");
            }
        };

        (is_full, capacity, is_same_element)
    }

    // check if the row is full or not
    // if the colum is full, check if all the element is same.
    fn check_row_full(&self, row_number: usize) -> (bool, usize, bool) {
        let row_to_check: [char; 4] = self.grid_without_wall[row_number];
        let mut capacity = 0;
        let mut is_full = false;
        let mut is_same_element = false;

        for i in 0..4 {
            if row_to_check[i] == '⬛' {
                capacity += 1;
            }
        }

        if capacity == 0 {
            is_full = true;
            if row_to_check.iter().all(|&x| x == row_to_check[0]) {
                is_same_element = true;
            }
        };

        (is_full, capacity, is_same_element)
    }

    // check if the diagnoal elements are same

    fn check_diagnoal_elements_same(&self) -> (bool, u8) {
        let mut is_same_element_diagonal = false;
        let mut winner_diagonal: u8 = 0;

        let diagonal_to_check_1: [char; 4] = [
            self.grid_without_wall[0][0],
            self.grid_without_wall[1][1],
            self.grid_without_wall[2][2],
            self.grid_without_wall[3][3],
        ];
        let diagonal_to_check_2: [char; 4] = [
            self.grid_without_wall[3][0],
            self.grid_without_wall[2][1],
            self.grid_without_wall[1][2],
            self.grid_without_wall[0][3],
        ];
        //check diagonal status

        if diagonal_to_check_1
            .iter()
            .all(|&x| x == diagonal_to_check_1[0])
        {
            if diagonal_to_check_1[0] != EMPTY {
                is_same_element_diagonal = true;
                winner_diagonal = 1;
            }
        }
        if diagonal_to_check_2
            .iter()
            .all(|&x| x == diagonal_to_check_2[0])
        {
            if diagonal_to_check_2[0] != EMPTY {
                is_same_element_diagonal = true;
                winner_diagonal = 2;
            }
        }

        (is_same_element_diagonal, winner_diagonal)
    }

    // check if the column is full or not
    fn get_game_status(&self) -> GameStatus {
        let mut game_status = GameStatus::game_on_going("Game is on going. Come on!".to_string());
        let mut winner: char = EMPTY;

        // check if the game is over when the column is full with same tile
        for i in 0..4 {
            info!("\n \nchecking column {} ", i);
            if self.check_column_full(i).0 && self.check_column_full(i).2 {
                info!(
                    "Column {} is full!  && Colum {} elements are identical.",
                    i, i
                );
                winner = self.grid_without_wall[0][i];

                game_status = GameStatus::game_over_with_winner(winner);
            }
        }

        // check if the game is over when the row is full with same tile
        for j in 0..4 {
            if self.check_row_full(j).0 && self.check_row_full(j).2 {
                info!("Row {} is full!   && Row {} elements are identical!", j, j);
                winner = self.grid_without_wall[j][0];
                game_status = GameStatus::game_over_with_winner(winner);
            }
        }

        // check if the game is over when the diagonal is full with same tile
        if self.check_diagnoal_elements_same().0 {
            info!("Diagonal elements are same. Game over with winner! :) ");
            if self.check_diagnoal_elements_same().1 == 1 {
                winner = self.grid_without_wall[0][0];
            } else if self.check_diagnoal_elements_same().1 == 2 {
                winner = self.grid_without_wall[3][0];
            } else {
                error!(
                    "please check winner_diagonal: {}",
                    self.check_diagnoal_elements_same().1
                )
            }

            game_status = GameStatus::game_over_with_winner(winner);
        }

        //GameStatus::on_going("Game is on going. Come on!".to_string());

        //GameStatus::game_over_without_winner("Game over without winner! :(".to_string());
        if (self.check_column_full(0).0 && !self.check_column_full(0).2)
            && (self.check_column_full(1).0 && !self.check_column_full(1).2)
            && (self.check_column_full(2).0 && !self.check_column_full(2).2)
            && (self.check_column_full(3).0 && !self.check_column_full(3).2)
        {
            info!("all columns are full! Game over without winner! ");
            game_status =
                GameStatus::game_over_without_winner("Game over without winner! :(".to_string());
        }

        //GameStatus::on_going("Game is on going. Come on!".to_string());
        info!("GameStatus is {:?}", game_status);

        game_status
    }



        //fn generate_random_grid(&mut self, rng: &mut rand::rngs::StdRng) {
            fn generate_random_grid(&mut self, rng: &mut rand::rngs::StdRng) {
                // 按行按列填充棋盘
                for row in 0..4 {
                    for col in 1..5 {
                        // 使用 rng 生成一个随机的 bool 值
                        if rng.gen::<bool>() {
                            self.grid[row][col] = COOKIE;
                        } else {
                            self.grid[row][col] = MILK;
                        }
                    }
                }
        
                self.grid_without_wall = self.get_grid_without_wall();
                info!("A random grid was created!");
            }


}

//actix_web response 1-1
#[get("/12/board")]
pub async fn handle_board(
    board_grid: web::Data<Arc<Mutex<Board>>>,
    body: String,
    req: HttpRequest,
) -> impl Responder {
    // output the post request
    info!("------> Received original body: \n{}", body);
    info!("------> Req is : \n{:#?}", req);

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("content_type is: {:?}", content_type);

    let mut board_string = "initial handle_board".to_string();
    let mut http_result_output = HttpResponse::Ok().body(board_string);

    //let counter = Arc::new(Mutex::new(0));

    let mut grid_lock_handle = board_grid.lock().await; // 获取锁

    match grid_lock_handle.get_game_status() {
        GameStatus::game_on_going(message) => {
            let board_string = grid_lock_handle.board_to_string();

            info!("board grid was updated. board_string is \n{}", board_string);
            http_result_output = HttpResponse::Ok().body(board_string);
        }
        GameStatus::game_over_with_winner(winner) => {
            let board_string =
                grid_lock_handle.board_to_string() + &winner.to_string() + " wins!\n";

            info!(
                "board grid is full now.   board_string is \n{}",
                board_string
            );
            http_result_output = HttpResponse::Ok().body(board_string);
        }
        GameStatus::game_over_without_winner(message) => {
            http_result_output = HttpResponse::Ok().body("tbd".to_string());
        }
    }

    http_result_output
}

//actix_web response 1-2
#[post("/12/reset")]
pub async fn board_reset(
    board_grid: web::Data<Arc<Mutex<Board>>>,
    body: String,
    req: HttpRequest,
) -> impl Responder {
    let mut grid_lock_reset = board_grid.lock().await; // 异步获取锁以修改数据
    grid_lock_reset.board_reset();
    let board_string = grid_lock_reset.board_to_string();

    info!("board grid was reset!!!");
    info!("board_string is \n{}", board_string);

    HttpResponse::Ok().body(board_string)
}

//actix_web response 1-1
#[post("/12/place/{team}/{column}")]
pub async fn place_cookie_milk(
    board_grid: web::Data<Arc<Mutex<Board>>>,
    path: web::Path<(String, String)>,
    body: String,
    req: HttpRequest,
) -> impl Responder {
    // output the post request
    info!("********************* Game Start! *************************************");
    info!("------> Received original body: \n{}", body);
    info!("------> Req is : \n{:#?}", req);
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("content_type is: {:?}", content_type);

    // 1. check if team and column is valid or not.
    let (team_input, column_input) = path.into_inner();
    info!(
        "Team: {}, Column: {}",
        team_input.clone(),
        column_input.clone()
    );

    let mut team: String = "default".to_string();
    let mut input_is_valid: bool = false;

    let column: usize = match column_input.parse() {
        Ok(num) => num, // 如果解析成功并且是正整数
        Err(_) => {
            return HttpResponse::BadRequest()
                .body("Column input must be a valid positive integer!")
        }
    };
    if column == 0 || column > 4 {
        info!("Column input should between 1 to 4.");
        return HttpResponse::BadRequest().body("Column must be a valid integer!");
    }

    let valid_teams = vec!["cookie", "milk"];
    if !valid_teams.contains(&team_input.as_str()) {
        info!("team input is not valid!!!");
        return HttpResponse::BadRequest().body("teams value is not valid ");
    }
    team = team_input;

    let mut board_string = "initial".to_string();
    let mut board_string_tail = "";
    let mut http_result_output = HttpResponse::Ok().body(board_string);

    let mut grid_lock_place = board_grid.lock().await; // 异步获取锁以修改数据


    //2 . check game status, if status  is game over, or ongoing
    match grid_lock_place.get_game_status() {
        GameStatus::game_on_going(message) => {
            // 3. before inserting item, check if the column is full or not
            info!("column is : {}", column);
            let (is_full, capacity, is_identical) = grid_lock_place.check_column_full(column - 1);

            info!(
                "==========> Column {} capacity is:  {} after check!!!!",
                column, capacity
            );

            // 4. insert the item
            if team == "cookie" {
                grid_lock_place.update_grid(capacity - 1, column, COOKIE);
                //grid_lock_place.grid[capacity - 1][column] = COOKIE;
                board_string_tail = "🍪 wins!\n";
            }

            if team == "milk" {
                grid_lock_place.update_grid(capacity - 1, column, MILK);
                //grid_lock_place.grid[capacity - 1][column] = MILK;
                board_string_tail = "🥛 wins!\n";
            }

            // 5. return the updated board and http status
            match grid_lock_place.get_game_status() {
                GameStatus::game_on_going(message) => {
                    let board_string = grid_lock_place.board_to_string();

                    info!("board grid was updated. board_string is \n{}", board_string);
                    http_result_output = HttpResponse::Ok().body(board_string);
                }
                GameStatus::game_over_with_winner(winner) => {
                    let board_string =
                        grid_lock_place.board_to_string() + &winner.to_string() + " wins!\n";

                    info!(
                        "board grid is full now.   board_string is \n{}",
                        board_string
                    );
                    http_result_output = HttpResponse::Ok().body(board_string);
                }
                GameStatus::game_over_without_winner(message) => {
                    info!("After placing {} in column {}", team, column);
                    info!(
                        "************** Game over! Without winner! *****************************"
                    );
                    let board_string = grid_lock_place.board_to_string() + "No winner.\n";
                    info!("board_string is \n{}", board_string);
                    http_result_output = HttpResponse::Ok().body(board_string);
                }
            }
        }

        GameStatus::game_over_with_winner(winner) => {
            let board_string = grid_lock_place.board_to_string() + &winner.to_string() + " wins!\n";

            info!("board grid is already full. game_over_with_winner  Servive unavailable.  board_string is \n{}", board_string);
            http_result_output = HttpResponse::ServiceUnavailable().body(board_string);
        }

        GameStatus::game_over_without_winner(message) => {
            info!("************** Game over! Without winner! *****************************");
            let board_string = grid_lock_place.board_to_string() + "No winner.\n";
            info!("board_string is \n{}", board_string);
            http_result_output = HttpResponse::ServiceUnavailable().body(board_string);
        }
    }

    return http_result_output

}



//actix_web response 1-1
#[get("/12/random-board")]
pub async fn random_board(
    board_grid: web::Data<Arc<Mutex<Board>>>,
    body: String,
    req: HttpRequest,
) -> impl Responder {
    // output the get request
    info!("------> Received original body: \n{}", body);
    info!("------> Req is : \n{:#?}", req);

    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    info!("content_type is: {:?}", content_type);

    let mut grid_lock_random = board_grid.lock().await; // 异步获取锁以修改数据
    let mut http_result_output = HttpResponse::Ok().body("initialize".to_string());

    //let mut rng = StdRng::seed_from_u64(2024);
    let mut rng = StdRng::seed_from_u64(2024);

    grid_lock_random.generate_random_grid(&mut rng);
    grid_lock_random.generate_random_grid(&mut rng);
    grid_lock_random.generate_random_grid(&mut rng);

    // 5. return the updated board and http status
    match grid_lock_random.get_game_status() {
        GameStatus::game_on_going(message) => {
            let board_string = grid_lock_random.board_to_string();

            info!("board grid was updated. board_string is \n{}", board_string);
            http_result_output = HttpResponse::Ok().body(board_string);
        }
        GameStatus::game_over_with_winner(winner) => {
            let board_string = grid_lock_random.board_to_string() + &winner.to_string() + " wins!\n";

            info!(
                "board grid is full now.   board_string is \n{}",
                board_string
            );
            http_result_output = HttpResponse::Ok().body(board_string);
        }
        GameStatus::game_over_without_winner(message) => {
            info!("************** Game over! Without winner! *****************************");
            let board_string = grid_lock_random.board_to_string() + "No winner.\n";
            info!("board_string is \n{}", board_string);
            http_result_output = HttpResponse::Ok().body(board_string);
        }
    }


    return http_result_output;
}