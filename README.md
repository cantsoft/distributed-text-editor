# Distributed Concurrent Text Editor
This is a repo for the group student project about creating a distributed concurrent text editor.

## Project Roles
* [Sławek Brzózka](https://github.com/Ezic04) -
    concurrent text synchronization, writing LaTeX reports and networking logic

* [Julian Konowalski](https://github.com/JulianKonowalski) - 
    GUI, networking logic and repository management

* [Jan Zadrąg](https://github.com/j4xz1) - 
    Rust backend tests

#  Building The Project

## Prerequisites
To build the project make sure you have installed:
* Rust with all of it's necessary components
* Node.js
* Protobuf compiler

## Building
To build the project, start off the command line in the project's root directory. Then you can use
these commands to build and run the project:
```
python scripts/run.py -i
```
to run the project with npm package preinstallation or
```
python scripts/run.py
```
to skip the npm package checks.