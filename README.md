# Distributed Concurrent Text Editor
This is a repo for the group student project about creating a distributed 
concurrent text editor.

## Project Roles
* [Sławek Brzózka](https://github.com/Ezic04) - concurrent text synchronization & writing LaTeX reports
* [Jan Zakroczymski](https://github.com/Balu46) - GUI & project management
* [Julian Konowalski](https://github.com/JulianKonowalski) - networking & repository management
* [Jan Zadrąg](https://github.com/j4xz1) - GUI & tests

#  Building The Project

## Prerequisites
To build the project make sure you have installed:
* Rust with all of it's necessary components
* Node.js

## Building
To build the project, start off the command line in the project's root directory. Then you can use
these commands to build and run the project:
```
# build rust package
cd backend
cargo build
cd ..

# setup js environment
cd frontend
npm install

# build the Node module and run the app
npm run build:backend
npm start
```