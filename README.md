# What Is This?

## Sand Drop Clicker
Based on the project name, you can already get an idea of what it's about, but I’ll thoroughly explain it to ensure no detail is missed. Sand Drop Clicker is about dropping sand into a container, and the amount of sand dropped will depend on the number of clicks the user makes. Grains of sand can be exchanged for currency, but the amount of sand sold cannot exceed the player's container size. The game will provide the user with upgrades in exchange for currency, which will be in dollars ($). These upgrades will range from the types of consumers of sand to personal upgrades, like the container sizes. Another interesting aspect of the program is that there will be an end goal, which is uncommon in many clicking games. The end goal will involve reaching 1 billion dollars, becoming the world’s greatest sand sales man. The end goal won’t reset your progress, so you’ll be able to gain even more wealth. 

## Who am I? And Why?

Hello my name is Artem and I (In this moment of time) am a student at Portland State University. This program is a term project for my Rust Programming Language class (CS423 for those interested). Hopefully, this program will be used to show how well I understand the language and if I know what I'm doing (please have mercy). 

## How The Program Works

This program runs on it's own window, and doesn't need a browser. 

The program uses __five__ specific Rust crates:

- ggez
- ggegui
- rand
- strum
- strum_macros

#### GGEZ & GGEGUI:

The ggez library is a lightweight game framework used for making 2D games. This crate handles the game logic and looping. The ggegui crate is an implementation of egui (another rust crate) for ggez, which prodivdes simple and fast gui. This handles the 'option' menu within the game.

#### RAND

The rand crate is for generating random numbers, which is used within the game whenever multiple grains sand are dropped. 

#### STRUM & STRUM_MACROS



## How To Run The Program

```sh
cd ./path/to/Sand-Drop-Clicker/
cargo run
```

## Lessons Learned

### Things That Didn't Work Out:



### What I learned:

