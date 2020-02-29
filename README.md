![a](logo.png)

TodoCList (from TodoList and Command Line Interface CLI) its a cmd & filesystem based TodoList app.

Unlike other Todo utilities, this doesn't require online connection of login. You only have to use the terminal and all the operations will be performed in a JSON file.

Since everything is file-based, you can also use a version control system to track your todo list among the project.

## Installing

You can install TodoCList downloading the binaries and placing them in a place where you terminal have access (or adding a enviorment variable in windows)

Also you can install it using cargo (the Rust building system) using:

`cargo install`

or (in newer versions)

`cargo install --path .`

## I don't want colors or they are not working

You can download the no_color binary or use:

`cargo install --features no_color`

or (in newer version)

`cargo install --features no_color --path .`

## How to use

First of all, you can to initialize your project:

`todoclist init`

Now you can add new tasks using:

`todoclist add "A new task"`

You can see the task list with:

`todoclist list (Your new tasks should be there)`

Now complete the task using (you should include the task id):

`todoclist complete 1`

Now the list will not show the task