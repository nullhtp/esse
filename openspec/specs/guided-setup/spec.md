# guided-setup Specification

## Purpose

The conversation that finishes an install: one command, carried by the bundle
and named by the installer, asking where the essays live, whether esse waits at
login, and which key brings it forward.

It answers beginner problem #2 ("cannot start") one step before the summon key
does. An install that ends in /Applications leaves three things for the writer
to carry — a key learned from caveats, a launch agent run by hand, a folder
discovered by opening the app — and each of them is a reason to not begin. This
capability is those three, asked once, in the terminal the writer is already in.

Everything here happens outside the app and stays there. A Homebrew cask cannot
prompt, so the questions belong to a command the install points at; and esse has
no settings, so the answers are recorded in a file the app only ever reads.
There is no preferences window, no folder picker and no shortcut recorder — the
anti-features list rules them out, and `first-run-onboarding` keeps the 3–5
sparks prompt as the entirety of what happens on first launch.

## Requirements
### Requirement: One command finishes the install
The installed bundle SHALL carry an interactive setup command, and the
installer SHALL name it in one line the writer can copy. Running it SHALL ask
exactly three things — where the essays live, whether esse waits at login, and
which key brings it forward — and SHALL end by saying, in one line, how esse is
now reached. It MUST need nothing from the source repository, and building from
source SHALL offer the same command.

#### Scenario: The install points at one line
- **WHEN** esse is installed with Homebrew
- **THEN** the caveats name the setup command as the one line that finishes the install

#### Scenario: Setup runs from the installed copy alone
- **WHEN** someone with no clone and no toolchain runs the setup command from the installed bundle
- **THEN** it asks its three questions and completes without any file from the repository

#### Scenario: It ends by saying how esse is reached
- **WHEN** the last question is answered
- **THEN** setup prints the combination that now brings esse forward, and the folder the writing will be in

### Requirement: Setup asks where the essays live
Setup SHALL offer the folder esse would open today as the default and SHALL
accept any other path, expanding a leading `~`, creating the folder if it does
not exist, and recording it. A path that cannot be created SHALL be reported
and the question asked again, rather than ending the setup.

#### Scenario: The default is taken
- **WHEN** the writer answers the folder question with `enter`
- **THEN** the writing stays where it is and no location is recorded

#### Scenario: Another folder is named
- **WHEN** the writer names `~/Writing/Essays`
- **THEN** the folder is created if needed, and every later launch of esse opens it

#### Scenario: A folder that cannot be created
- **WHEN** the named path cannot be created
- **THEN** setup says why and asks the question again

### Requirement: Existing writing is offered a move
When writing already exists where esse currently opens and a different folder
is named, setup SHALL say how much is there — essays and sparks — and SHALL
offer to move it whole. Declining SHALL leave every file where it is and say
so out loud. Setup MUST NOT merge two folders: if the named folder already
holds esse's files, it SHALL move nothing and say which two folders it found.
A move that fails SHALL leave the writing at its original location and record
no new location.

#### Scenario: The move is accepted
- **WHEN** the writer names a new folder, is told the old one holds 12 essays and 40 sparks, and accepts the move
- **THEN** every file is in the new folder, nothing is left at the old path, and esse opens the new folder from then on

#### Scenario: The move is declined
- **WHEN** the writer declines the move
- **THEN** the old folder is untouched, setup says the earlier writing stayed behind and where it is, and esse opens the new folder

#### Scenario: Two folders are never merged
- **WHEN** the named folder already holds esse's files and so does the current one
- **THEN** setup moves nothing, names both folders, and asks the question again

### Requirement: Setup asks whether esse waits at login
Setup SHALL ask whether esse should start at login holding its key, defaulting
to yes on a machine that has never been set up, and otherwise to what is
currently true — a launch agent that is installed, or a refusal recorded by an
earlier run. Answering yes SHALL install the launch agent and start it now, so
the key answers without another launch. Answering no SHALL remove the launch
agent if one is installed, SHALL be remembered so that a later run does not
quietly undo it, and SHALL leave the writing untouched.

#### Scenario: Autostart is turned on
- **WHEN** the writer accepts the login question
- **THEN** the launch agent is installed and running, and the summon key answers immediately without opening the app by hand

#### Scenario: A later run defaults to what is true
- **WHEN** setup is run again on a machine where esse already starts at login
- **THEN** the login question offers yes as its default

#### Scenario: Autostart is turned off
- **WHEN** the writer declines the login question on a machine where esse starts at login
- **THEN** the launch agent is removed, esse no longer starts at login, and the writing is untouched

#### Scenario: A refusal is not quietly undone
- **WHEN** setup is run again on a machine where the writer declined the login question earlier
- **THEN** the login question offers no as its default, and answering it with `enter` leaves esse not starting at login

### Requirement: Setup says the summon key and takes another
Setup SHALL print the combination that will bring esse forward and SHALL accept
another one, spelled the way the app spells keys everywhere else, keeping the
current combination when the answer is empty. A spelling the app cannot parse
SHALL be reported and the question asked again. A combination named here SHALL
be recorded and SHALL also be written into the launch agent, so that a login
launch and an ordinary launch answer the same key.

#### Scenario: The key is told, not documented
- **WHEN** setup reaches the key question
- **THEN** it prints the combination that brings esse forward

#### Scenario: The default combination is kept
- **WHEN** the writer answers the key question with `enter`
- **THEN** `ctrl-alt-e` stays the combination and none is recorded

#### Scenario: Another combination is named
- **WHEN** the writer names `cmd-shift-space`
- **THEN** that combination is recorded, is written into the launch agent if one is installed, and answers on the next launch however esse is started

#### Scenario: A spelling esse cannot read
- **WHEN** the writer names a combination the app cannot parse
- **THEN** setup says so and asks the question again, and nothing is recorded

### Requirement: The answers are recorded where every launch reads them
Setup SHALL record its answers in one file outside the writing, and the app
SHALL read it at startup so that a launch from Finder, from the Dock-less
summon key and from the launch agent all agree. The file SHALL record only the
answers that depart from the defaults, and SHALL be removed when none of them
does. The app MUST NOT write this file, and MUST NOT ask
these questions in a window: there is no settings screen, no preferences pane
and no folder picker anywhere in the app. Uninstalling esse SHALL be able to
take this file with it, and SHALL never take the writing.

#### Scenario: A named folder holds on an ordinary launch
- **WHEN** a folder was named at setup and esse is later opened from Finder
- **THEN** esse opens that folder, and the Shelf says that path at its foot

#### Scenario: A named key holds on an ordinary launch
- **WHEN** a combination was named at setup and esse is later opened without the launch agent
- **THEN** that combination is the one that brings esse forward

#### Scenario: Every default leaves nothing behind
- **WHEN** setup is run and every question is answered with its default
- **THEN** no answers file exists, and the app behaves exactly as it does with no setup ever run

#### Scenario: The app never writes the answers
- **WHEN** esse runs, writes essays, publishes and quits
- **THEN** the answers file is exactly as setup left it

### Requirement: Setup is safe to run again
Running setup a second time SHALL show what is currently true as the default
for every question and SHALL change only what the writer changes. Answering
every question with `enter` SHALL leave the folder, the launch agent, the
recorded answers and the writing exactly as they were.

#### Scenario: Enter all the way through changes nothing
- **WHEN** setup is run again and every question is answered with `enter`
- **THEN** the data folder, the launch agent and the answers file are unchanged

#### Scenario: One answer changes, the others hold
- **WHEN** setup is run again and only the key question is answered differently
- **THEN** the new combination is recorded and the folder and the launch agent's on-or-off state are unchanged

### Requirement: Without a terminal, setup changes nothing
When setup is run with no terminal to answer it — its input is not a TTY — it
SHALL print the answers that are currently in force, SHALL change no file, no
launch agent and no writing, and SHALL exit reporting success.

#### Scenario: Run from a script
- **WHEN** setup runs with its input redirected from a file or a pipe
- **THEN** it prints the current folder, the current combination and whether esse starts at login, exits 0, and installs no launch agent

