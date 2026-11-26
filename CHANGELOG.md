# Changelog

All notable changes to FM ValueScout are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.39.0] - 2026-11-26

### Added

- **FM's own faces for newgens.** A player FM generated now shows the face FM shows for him when **Use pre-generated images of players and staff** is on, wherever the app shows a portrait and your graphics folder has no image for him. The app works the face out with FM's own rule and reads the picture from the portrait file of your FM install, so an installed face mod is what you see. An image from your graphics folder still comes first, as in FM, and the faces also show with no graphics folder chosen. Real players without a photo and staff keep the placeholder. The first face after the app starts takes a few seconds. The faces were checked against FM 27.1.1 with the Photoreal Faces V2 mod; an install whose portrait file is in another format shows the placeholder.

### Changed

- **This release needs the new bridge plugin.** Use **Install / Update plugin** in Settings, restart FM, and Load Data with FM open on your save. Load Data does not work with the older plugin, and a snapshot loaded before this update shows no game faces until you load data again.

## [0.38.0] - 2026-11-26

### Added

- **Development slots.** Each player at your club up to an age limit, 23 by default, now has a development slot: the slot of your tactic where his potential compares best with the other players in the save. On **Training**, a **Develops as** column names it by position and in-possession role, and you can choose another, including a position he has yet to learn. A **Developing towards** strip counts the players growing into each slot and marks the slots nobody is growing into. **Develop players aged up to** sets the age limit. Training has its own **Position familiarity** switch, kept with the save, which decides whether a slot is suggested only among the positions a player can play or in any position.
- **Train for a development slot.** **Train for** on Training switches the save between **Planner slot**, as before, and **Development slot**, where each developing player trains the roles of his development slot, a developing player in no squad included, and older players train the slot they play. Squad's Suggested Training follows the same choice.
- **Optimize for development.** **Optimize for** in the Planner chooses **Strongest**, as before, or **Development**, which places a developing player only in his development slot and says what the starters give up. The board marks a developing player who plays away from his slot.

### Changed

- **A quieter interface.** Most explanatory text is gone from the screens. An explanation now sits in a tooltip on the control it explains, a finished action shows a tick on its button, and a clean Load Data result is one line with the rest behind **Details**. Errors, warnings, and anything that cannot be undone stay visible.
- **Controls in the app's own style.** Checkboxes, drop-down lists, the tactic's weight slider, progress bars, and number and search fields no longer use the system's look. On/off options such as **Position familiarity**, **Season stats**, **Only realistic signings**, **Shortlist**, and **Only unemployed** are switches. Icon buttons show their name in the app's own tooltip.
- **Less memory and a faster start.** On a 10 GB database with a large graphics pack, the app at rest uses about 210 MB where it used about 750 MB. It reaches a loaded Dashboard in about 4.5 seconds where it took about 8, and in about 11 seconds where it took 26 when Windows has not read the database recently. After 30 seconds without activity the app frees the memory it is not using.
- **A smaller database and faster season statistics.** Season statistics are stored more compactly: a 9.7 GB database shrank to 5.5 GB, and sorting Search by a statistic takes less than half as long. The first start after this update takes several minutes, because the app first converts the stored statistics and then shrinks the database file. The window stays blank and Windows may mark it "Not Responding" until both steps finish, so leave it running. If the app is closed during the conversion, the conversion runs again on the next start. If it is closed while the file is shrinking, or the disk has less free space than about twice the size of the shrunken database, the data is safe but the file keeps its old size for good.

## [0.37.0] - 2026-11-24

### Added

- **Training page.** **Training**, between Tactic and Youth, lists the individual training each player at your club would get in FM, under one heading per squad string: his slot, the in-possession role and its position, the out-of-possession role, and the focus. A filter shows all players, one squad, the players in no squad, or the players learning a position.
- **Training for a position a player is still learning.** FM builds position familiarity only where the in-possession role is trained, so the plan now reads each player's familiarity with his Planner slot's two positions. A player below 16 of 20 at the slot's in-possession position trains the slot's own role there and is marked **Learning** with his familiarity. A player who is familiar there but below 16 at the out-of-possession position trains an in-possession role at that position instead: the slot's own role where FM has it there, otherwise the closest one by attributes. He goes back to the slot's own training once a later Load Data shows him familiar. This suits squads optimized with **Position familiarity** off.

### Changed

- **Apply training in FM moved to Training.** It was in Squad. It sets the training of the ticked players in the current filter, so you can apply one squad or only the players learning a position, and its result stays above the table. The list follows the Planner: squad, string, and slot, then the players in no slot by name.

## [0.36.0] - 2026-11-24

### Changed

- **Smoother navigation.** A screen now opens once, in its final layout. The main screens load in the background after the app starts and after Load Data and stay ready for 30 minutes, so most navigation shows no loading state. Where a screen still waits, a placeholder in the shape of its content holds the space, and nothing moves when the data arrives. A sort, filter, or column change keeps its table in place and dims the rows until the new order arrives. The page fades between screens, menus and pickers ease in, and a thin bar on the navigation shows when a screen takes more than a moment to open. The window opens dark with a dark title bar.
- **Faster screens on large saves.** Squad, My Staff, the club picker, a player's Moneyball tab, Search sorted by a Moneyball Score, and the Dashboard's expiring contracts and prospects now read from indexes where they scanned the whole snapshot. On a save with 170,000 players, Squad and My Staff open in a few milliseconds, and a Moneyball sort in Search takes about half as long.
- **Recommended signings are scored once.** The Dashboard scores them after your data, tactic, Planner, or shortlists change, and answers later visits and slot changes from that without scoring again. The scoring itself is two to three times faster in Moneyball mode.
- **Portraits and crests stay loaded.** A table row that scrolls back into view shows its portrait and crest without loading them again.

### Fixed

- **A growing database log slowed every screen.** On Windows the app could not finish folding its write-ahead log into the database after Load Data or at exit, so the log kept growing, to 14 GB on one save, and every screen read through it. The app now finishes that step after each Load Data, at exit, and at start. The first start after this update takes a minute or two before the window opens, because the app folds in the old log and builds the new indexes once.

## [0.35.0] - 2026-11-24

### Added

- **Optimize squads without position familiarity.** Turn off **Position familiarity** under **Optimize squads** in the Planner to place players in any position by their role scores and preferred foot alone, and see where each player's attributes fit best. Pins, age limits, and every other rule still apply, and goalkeepers and outfield players never swap. Optimize then says how many players are out of position. The board marks each player who cannot play his slot, including one you placed yourself. Reserves and Youth still place each player in the slot he stands out in, so one slot can hold many of them. The checkbox is on again when the app restarts. The Dashboard still needs position familiarity, so it shows an out-of-position Planner pick as having no score there.

### Changed

- **Saved filters add your recruiting defaults.** Applying a saved filter or a tactic slot's filters from **Saved filters** in Search also adds **Age ≤ 28** and, when your club is set, **Realistic signing**. Remove either chip to search without it. A filter with its own age or realistic-signing rule keeps it, and a filter that matches **Any** of several rules gains neither.
- **Recommended signings count age.** Players over 28 lose points for each year: in Moneyball mode 1 at 29, 3 at 30, 6 at 31, 10 at 32, and 15 at 33, and in General mode half of that, rounded up. The score, gain, and order in **Recommended signings** and **Best per slot** use what is left, so an older player is recommended only for a large enough improvement. A card shows the points lost, such as **Age −5**, beside the gain. Shortlisted players lose points too; scores everywhere else are unchanged.
- **Youth staff recommendations** weigh Working with Youngsters at 40%, down from half. Reserves, Senior, and club-wide jobs are unchanged.
- Updated compatible Rust dependencies.

### Fixed

- **Season stats on more saves.** Load Data and **Refresh stats** no longer reject a save because a few players' overall totals do not match the sum of their competitions. A save that fails a data check now says so and is no longer reported as an unsupported FM27 build.

## [0.34.0] - 2026-11-21

### Added

- **Saved filters can set Search's columns.** Applying a saved filter or a tactic slot's filters from **Saved filters** now asks whether to replace the columns. **Replace columns** shows Age, Nationality, Value, the matching role scores, and a column for each field the filter uses, such as Height, Preferred foot, or a statistic. Each role the filter scores on gets both its Role Score and its Potential Role Score; a Moneyball Score rule shows only its own column. A tactic slot also shows its own Role Score and Potential. If you were sorting by a tactic column the new layout drops, the sort returns to the default. **Keep columns** adds the filter's fields to the current columns, as before, and Escape cancels the apply. Tick **Don't ask again this session** to keep your answer until the app restarts. Search does not ask when the columns already match.

## [0.33.0] - 2026-11-21

### Added

- **Recruit each team's staff from its own shortlist.** Put "senior", "reserve", or "youth" anywhere in an FM staff shortlist's name, in any case (such as "Reserves Staff"), and the assignment optimizer recruits for that team only from that list. A team without its own list recruits only people on no team's list, so a coach on the senior list never goes to the youth team. Club-wide jobs still recruit from every shortlist, current club staff can still fill any team, and saves without team lists are unchanged. The reasons shown for an unfilled slot count only the people that team may take. Load Data again to read the team lists; a team list with nobody Load Data could read counts as no list.

### Changed

- **Boost all CA paces players towards their PA by age 28.** My Club's **Boost all CA** and the player step of **Apply all boosts** in Settings now add the gap between a player's CA and PA divided by the seasons left until 28, rounded up, and never more than +20. Players aged 27 or 28 count as one season left, and players aged 29 or older are still skipped. Run it once a season: each run takes a share of the gap that is left, so more frequent runs add up. The **Boost CA** button on a player's profile still adds +5 up to age 20 and +10 from 21 through 28. This needs the new bridge plugin: use **Install / Update plugin** in Settings and restart FM. The old plugin fails every boost other than +5 or +10.

## [0.32.0] - 2026-11-20

### Added

- **Apply your Planner squads in FM.** **Apply in FM** beside **Optimize squads** moves your players into the FM teams their Planner squads stand for: Senior to the first team, Reserves to the reserve team, and Youth to the youth team. It lists every move, all ticked, marks a move into a squad the player is too old for with **Over age**, and then reports what FM moved and why any move failed. FM makes each move itself, as its **Move to** menu does, and does not check age.
- **Apply the staff optimizer in FM.** **Apply in FM** in the assignment recommendations makes the optimizer's changes in FM: it sacks surplus staff, hires each recruit into the slot's job and team, and changes someone's job or team by sacking and hiring them again, because FM cannot change an employed person's job. FM sets every new contract and wage for the job, as the in-game editor's **Move to my club** does. The dialog lists each change with the person's job or club now, what they become, and their current wage, then reports each new job, team, and wage. FM checks neither the board's staff limits nor whether a job suits the person. If FM refuses a re-hire, that person is left unemployed, and the report names them.
- **Apply training in FM.** **Apply training in FM** in Squad sets each player's individual training in FM. A player in a Planner slot trains the slot's in-possession role at its position and its out-of-possession role, and every player's additional focus becomes his Suggested Training, cleared when he has none. Players out on loan are left out. The dialog reports how many players FM took and, for any it did not, what FM holds instead.
- **Staffing needs from FM.** Load Data reads the board's staff limits for your first, reserve, and youth teams from FM and sets the staffing needs from them. The first Load Data sets every need FM has a limit for; after that, a need you edit stays until FM's limit for it changes. **Configure staffing needs** shows FM's limit beside any need that differs from it. The Load Data message says how many needs changed, or why the limits were not read.
- Everything in this release except the FM27 Suggested Training and the Set Piece Coach fix needs the new bridge plugin: use **Install / Update plugin** in Settings, restart FM, and Load Data with FM open on your save.

### Changed

- **Your club follows FM.** Load Data makes your FM manager's club the managed club, replacing an earlier pick. Choosing a club in My Club still works when Load Data cannot tell, and lasts until the next Load Data that can.
- **Suggested Training uses FM27's focuses.** It suggests from FM27's additional focus list, such as Chance Conversion, Ball Progression, Pressing, Resistance, and the new goalkeeper focuses, each with the attributes FM lists for it. A focus is ranked by its gain per attribute it trains, so a broad focus no longer wins just by training more attributes. Many suggestions change.

### Fixed

- Load Data places players in their FM team even when the team lists more than 60 players.
- The assignment optimizer keeps or replaces a current Set Piece Coach like any other job.

## [0.31.1] - 2026-11-20

### Fixed

- **Teams and loans on FM27.** Load Data places players in their FM teams again. Team level, loans, and club reputation, blank on FM27 until now, are filled in, and a player's league is his own team's rather than his parent club's. A B club such as Real Madrid Castilla counts as its parent club's reserve team. Moneyball adjusts B-team and loaned players for the league they play in now. Use **Install / Update plugin** in Settings and restart FM to get the new plugin; Load Data needs it.
- **Staff teams in assignment recommendations.** **Keep** now means the same job for the same team, read from each staff member's FM contract. Someone in the job for another team gets **Move team**, and **Change job** names the current team too, using your Planner team names.

### Changed

- U21 and U23 teams count as reserve teams, and U20 and younger teams as youth teams.
- Players you have loaned out stay listed in Squad, Youth classes, and expiring contracts, tinted steel and marked as on loan; Squad and Youth classes name the borrowing club. They get no Suggested Training, and the Planner, its Optimize, loan candidates, and the Dashboard's starting XI and prospects leave them out. Players on loan at your club count, but are never loan candidates.
- Assignment recommendations name whom each newcomer replaces, or that the job gains a place, and **Staff to sack** names each person's team and replacement.

## [0.31.0] - 2026-11-19

### Added

- **Staff to sack.** Assignment recommendations list your staff who lost every slot they could fill to someone better, with each person's current job, so the cuts are quick to make in FM.

### Changed

- **Optimize assignments** opens its recommendations in a dialog instead of a panel on Staff Search. Each recommendation says what to do: **Keep**, **Change job** (naming the person's current job), **Your staff**, or **Recruit** (naming their club, or Unemployed). Names open the Staff Profile, and the table scrolls with the dialog while its header stays in view.
- Unfilled slots are summed up in one notice with **Adjust staffing needs** and **Review shortlist**, and each unfilled slot says in plain words why no one fits.
- Staffing needs are set in two grids, **Team jobs** by team and **Club-wide jobs**, each with totals. Your own First Team Manager slot reads **You**. **Optimize assignments** with no staffing needs opens the grids, and saving them from there, or from **Adjust staffing needs** in the results, optimizes again.
- The optimizer's hints and messages use plain language instead of its internal terms.

### Fixed

- **Clubs that share a name.** FM27 gives some clubs the same name, such as Manchester United and Manchester United Women. Staff and players of a namesake club no longer count as your club's own in My Staff, the My Staff CA boost, the assignment optimizer, Squad and its boosts, the Planner, Youth, and the Dashboard. The optimizer offers a shortlisted person from a namesake club as a recruit, and the Dashboard counts a player at, or on loan from, a namesake club as a signing. A club chosen in My Club before Load Data recorded club identities (0.19.0) still matches by name; choose it again to fix that.
- Hovering a navigation link, or sorting and filtering Staff Search, no longer makes the app seem to refresh: the optimizer keeps its results, and My Club no longer blanks its squad overview.

## [0.30.0] - 2026-11-18

### Added

- **One Player Search.** Search and Moneyball are now one page. Filters, columns, and sort offer FM attributes, role scores, and Club DNA beside playing time, season statistics with their percentiles, and Moneyball scores, so one search can combine scouting and performance, such as PA 150 or higher with xG / 90 in the top fifth. Every player is listed. A player needs 450 minutes of season stats, this season or in a kept snapshot from the last year, to have statistics, percentiles, and Moneyball scores, and drops out of any filter on them otherwise. Playing-time filters count a player who has not played as 0 minutes.
- **Saved filters** beside **Add filter** save the applied filters, how they combine, and **Percentiles vs** under a name, to apply, rename, or delete later. They also list each tactic slot's recruitment filters: applying one keeps players rated 12 or higher at the slot's positions and ranks percentiles against the slot's two position groups, so Search lists the players the Dashboard compares for that slot.
- **Add Tactic (Moneyball Score)** shows each slot's Moneyball score, the one the Dashboard compares in Moneyball mode: from the slot's chosen metrics, or from its roles when it has none. With **Add Tactic (Role Score)** also on, each slot's two scores sit side by side.
- **Percentiles vs** can rank against two position groups together, such as wingers and wide midfielders.

### Changed

- The two kinds of role score are named apart everywhere: **Role Score** (and **Potential Role Score**) from attributes, **Moneyball Score** from season stats, including filter chips and Squad's role columns.
- Search uses season stats from kept snapshots as the Dashboard and the profile do. Values from a kept snapshot read muted, with their date on hover.
- **Stats as of**, **Percentiles vs**, and **League-adjusted | Raw** appear only while a season column, filter, or sort is in use. **Refresh stats** is always in Search's header.
- Clicking a Search row opens the profile on the view chosen in Settings, which now sets the default for the profile and the Dashboard.

### Removed

- The separate Moneyball page and its navigation link; old Moneyball links open Search. Its saved columns are not carried over, and Search keeps its own.
- **Add Tactic (Roles)** and **Add Tactic (Custom)**, replaced by **Add Tactic (Moneyball Score)**.
- Muted short-sample percentiles. A player under 450 minutes without kept stats has none.

## [0.29.1] - 2026-11-17

### Fixed

- **Load Data on FM 27.1.** Load Data failed on FM 27.1 because the bridge refused any game version other than 27.0. The bridge now reads every FM27 patch and checks each scan against the data it reads instead of the game version. If the players and staff no longer look like FM data, for example CA above PA across the database, Load Data stops, names the fields that moved, and keeps the previous snapshot. Use **Install / Update plugin** in Settings and restart FM to get the new plugin.

### Changed

- Player and staff boosts are no longer limited to one exact FM build. They are available on any FM27 patch once a Load Data of the running game passes that check. Back up the save before the first boost after a game patch.

## [0.29.0] - 2026-11-17

### Added

- **Season stats from kept snapshots.** A player with under 450 competitive minutes this season is now scored on his season stats from the latest kept snapshot in which he has 450 or more, when those stats are less than a year older than the current snapshot's in-game date. They are ranked against that snapshot's players and adjusted for the league he played in then. Recommended signings, Best per slot, and the starting XI use them, so early in a season a candidate is no longer held back for too few minutes when last season's stats are kept. Signing cards date borrowed minutes, and the Moneyball profile names the kept snapshot's date and this season's minutes. Moneyball Search still shows this season's stats only.

### Changed

- Snapshots keep their season stats after a newer load replaces them as current, until they are deleted. Keep a snapshot before the next Load Data to lend its stats later; each kept snapshot with stats takes more disk space. Snapshots kept before this release have no stats to lend.
- Updated compatible Rust dependencies.

## [0.28.0] - 2026-11-15

### Added

- **Recruitment filters** for each tactic slot: up to ten Search-style rules on a slot's **Recruitment** tab, over player fields such as foot, height, and role scores, and Moneyball statistics compared by value or by percentile. Recommended signings and Best per slot only suggest a player for the slots whose filters he passes; your own players are never filtered. The Dashboard names the selected slot's filters, says how many candidates they ruled out, links to edit them, and marks filtered slots in the starting XI.
- **Season stats** on Recommended signings. On, a candidate needs 450 competitive minutes this season and slots' statistic filters apply; off, both are skipped. It is always on in Moneyball mode and unavailable until the save has season stats.
- **Percentiles vs** in Moneyball Search ranks each player's statistics against the players of his own position (the default) or against one position group, such as centre-backs or wingers. A player under 450 minutes shows muted percentiles.
- The Moneyball profile names the position group its percentiles compare him with and, for a player who plays several, switches between them. It notes a short sample and a group with no qualifying players yet.

### Changed

- Every Moneyball percentile now compares a player with one position group: players with 450+ minutes this season rated 16 or higher at that group's positions, such as full-backs (left and right back) or wingers (left and right attacking midfield). A role compares with its own position group, a slot's custom metrics and percentile filters with the groups of its two positions, and a player's own statistics with his best position's group, so a role score is the same on the profile, in Search, and on the Dashboard. Moneyball scores in existing saves change accordingly; no stats refresh is needed.
- Search filters and shortlists now only choose which players are listed; they no longer change the players a percentile compares with.
- The profile shows Moneyball role scores for every player with season stats, including one without a natural position.
- The Tactic page fits a maximized 1080p window: the Tactical XI lists both roles of each slot in full with its metric and filter counts, pitch markers show positions only, and the wider Selected Slot panel has **Roles**, **Scoring**, and **Recruitment** tabs.
- **Refresh stats** and the season-stats step of Load Data are faster, because they no longer rank every player.

### Removed

- The **Filtered cohort | All players** toggle in Moneyball Search, replaced by **Percentiles vs**.
- The slot importance rank in the Tactic editor. Optimize and the Dashboard's starting XI no longer use a stored rank.

### Fixed

- Search could not add a percentile filter on a counted Moneyball statistic such as Goals or Progressive Passes.

## [0.27.1] - 2026-11-14

### Fixed

- **Only realistic signings** in Search and Moneyball no longer lists your own club's players, who cannot be signed. They are listed as before when the filter is off.

## [0.27.0] - 2026-11-14

### Added

- **Loans (N)** in the Planner opens a list of your players aged 18 to 23 outside the senior squad, ranked by how much a loan would help each one. The benefit weighs how much he is expected to grow, how close he is to the senior squad in his slot, and how little he would play at your club. The button counts the recommended players, and the board marks them with an icon. A player already out on loan may be listed, because loans cannot be read from FM27.

### Changed

- **Optimize squads** builds a senior squad of 30. The senior squad's strings are now **Starter**, **Backup**, and **Rotation**: the best lineup, the next best, and eight more players, one of them always a goalkeeper and no two for the same slot.
- Reserves and Youth are no longer picked as the strongest lineups. A player outside the senior squad joins the team with the lowest age limit that takes him, in the slot he stands out in: where his score ranks highest against every player in the save who can play that slot, not where his score is highest. Each slot lists its players best first, and a slot nobody stands out in stays empty.
- **Best role fit** lists each player under the slot he stands out in, by the same ranking.

### Removed

- The controls that add, rename, reorder, and remove Planner strings. The senior squad's three strings are fixed, and **Optimize squads** sets how deep Reserves and Youth go. On upgrade, a senior string after the third is deleted with its assignments, and custom string names in Reserves and Youth return to **1st string**, **2nd string**, and so on.

## [0.26.0] - 2026-11-13

### Added

- **Only realistic signings** in Search and Moneyball keeps the players your managed club could plausibly sign. It is an estimate from each player's ability, potential, and reputation against your squad's level and your club's reputation, not Football Manager's own check. It needs a managed club, chosen in My Club.
- The staff assignment optimizer also considers the staff already at your club, by the job each holds now. Your own manager is never suggested.
- **Skip** on a staff recommendation leaves that person out of every assignment for the save and optimizes again. **Skipped staff** lists them with **Restore**.
- Snapshot history has **Keep** and **Don't keep** for each snapshot and marks a kept one with a **Kept** chip.

### Changed

- The Dashboard's Recommended signings compare your club's realistic signings by default, so recommendations no longer need a shortlist. **Candidates** switches to one of your shortlists, whose players are compared as they stand. In Moneyball mode only players with season stats are compared.
- Load Data replaces the save's snapshots unless you keep them. Every snapshot stored before this version is kept, so upgrading deletes none.

### Removed

- **Add FM Shortlist** and linked FM shortlist files (`.fmf`), because Load Data and **Refresh stats** read your shortlists from the linked FM save. Shortlists that were linked to files are deleted on upgrade.
- The Dashboard's **All shortlists** option. Choose **Realistic signings** or one shortlist.

## [0.25.1] - 2026-11-11

### Fixed

- Load Data and **Refresh stats** read staff shortlists that 0.25.0 rejected with "this FM27 build is not supported yet". The FM27 build was not the cause: the save stores some staff in ways 0.25.0 did not read.
- One shortlisted staff member the app cannot read no longer drops the whole staff shortlist. That person is left out, and the Load Data banner says how many staff were read and how many were not.

## [0.25.0] - 2026-11-11

### Added

- Moneyball role and tactic scores account for league strength. Load Data measures each league from the current ability of its clubs' best elevens, and a player's scores move by half a point for every point his league is weaker or stronger than your managed club's league, or than the strongest league when you manage no club. This applies in Moneyball Search, in the Moneyball Profile's role fit, which names both leagues and the change, and in the Dashboard's Moneyball signings. Statistics and percentiles are unchanged. Run Load Data once to measure the leagues.
- **Scores: League-adjusted | Raw** in Moneyball Search turns the league adjustment on or off for the save.
- Staff Search reads your staff shortlists from the linked FM save. Load Data and **Refresh stats** read who is listed, the job each person prefers, and their coaching qualifications, so nothing has to be exported from the game. The Load Data banner says how many staff were read. When a person rates several jobs equally highest, the Preferred Job shown can differ from FM's.

### Changed

- **Only unemployed** in Staff Search now lists shortlisted staff who have no club in the loaded data.

### Removed

- The staff shortlist CSV upload and the Club Job column, because FM27 cannot export staff views. Staff imported from CSV earlier stay on the shortlist until Load Data reads the staff shortlists of a linked FM save.

### Fixed

- A named staff shortlist in the FM save no longer hides that save's player shortlists.

## [0.24.0] - 2026-11-11

### Added

- The Dashboard shows your managed club at a glance: the starting XI for your tactic, signings recommended from your shortlists and ranked by how much they improve each slot, contracts that end within 12 months, and your development prospects. Recommended signings can use General or Moneyball scores. **Best per slot** lists the best shortlisted signing for every slot, and **Youth XI** shows your best young players in your tactic's shape. A save without a managed club chooses one on the Dashboard.
- Load Data and **Refresh stats** read your player shortlists from the linked FM save, so a shortlist no longer has to be exported from the game first. Search and Moneyball label each shortlist with its source.
- Moneyball shows pressing, games won, drawn, and lost, win ratio, xG overperformance, and goalkeeper penalty and expected save statistics again, read from the linked FM save. The ten pressing roles score from all their metrics again. Run **Refresh stats** or Load Data once to fill them in.
- Reserves and Youth each take an oldest age in the Planner's **Manage teams**, from 14 to 45 or none. **Optimize squads** never places a player over it. Existing saves keep 23 and 18.
- The Planner's **Unassigned** button counts the players in no squad and lists them with the slot each fits best, and **Youth XI** opens from the Planner as well.
- Planner board cells show each player's age, a pin for a player you placed or pinned, and a warning for a player older than his team's age limit.

### Changed

- **Optimize squads** now weighs development. Each squad values a player by his score now plus a share of the score he is expected to reach, judged from his age, Professionalism, Ambition, and Determination. A prospect can start ahead of a slightly better veteran, and the Reserves and Youth squads favour the higher ceiling. Senior first and second strings fill first, then Reserves and Youth starters before deeper strings.
- Planner board cells show `Current → Expected` scores on one line with the player's name and age. The potential score is in the tooltip.
- Every Planner cell opens the same picker. For a placed player it pins or unpins him, clears the slot, or replaces him in one step. Candidates show their age and expected score, and say when an unfamiliar position, the wrong foot, or the age limit counts against them.
- Planner strings are added, renamed, reordered, and removed on the board. **Manage teams** now sets each team's name and age limit, and no longer edits strings.
- The Planner's squad controls are ordered from most to least used: **Optimize squads**, the read-only views, then **Manage teams** and **Clear all**.
- Moneyball's "xG Open Play" is now "xG Overperformance".

### Removed

- The Planner's **Optimize by potential**. **Optimize squads** now accounts for each player's expected development.

## [0.23.0] - 2026-11-10

### Added

- Each save can link its FM27 save file in Settings. Load Data reads that file's season statistics for Moneyball, and **Refresh stats** in Moneyball Search re-reads it on demand. Moneyball Search and the Moneyball Profile show the date the stats are from.
- Each save can keep several linked FM player shortlist files (`.fmf`). **Add FM Shortlist** links a file, a **Shortlist** select in Search and Moneyball switches between them, and Load Data re-reads every linked shortlist.
- Moneyball Search can be restricted to the selected shortlist, including its Filtered cohort percentiles.
- Moneyball metric filters can compare by percentile as well as by value.
- The Tactic editor can choose up to five Moneyball metrics for each slot, and Moneyball Search adds **Add Tactic (Custom)** columns that score players by those metrics.
- Moneyball Search can filter, sort, and show CA, PA, reputation, wage, contract, transfer and loan, and team level fields.
- Settings lists open-source notices.

### Changed

- Filters in Search and Moneyball are edited directly from their chips, with **Match All | Any**, range and multi-choice operators, and percentage statistics compared on their displayed scale. Columns and field pickers share one grouping, sorting is available from column menus and the sort summary, and each view remembers its filters and sort for the session.
- Search, Squad, and Staff keep their rows visible while a sort, filter, or column change loads, and scrolling large tables is smoother.
- Large saves load and search much faster, the window stays responsive during Load Data and long searches, and Load Data uses much less memory.
- Opening a player's Moneyball profile is faster and no longer freezes the window.
- The Moneyball comparison pool **Full CSV** is now **All players**.
- My Club keeps Planner and Tactic drafts when you switch between workspaces.
- The window enforces a 1280×800 minimum size, and interface copy, icons, cursors, headings, and table headers are more consistent.
- The Dashboard shows an empty state, and unexpected errors show a page with **Retry**.
- Updated compatible Rust and frontend dependencies.

### Removed

- The Moneyball CSV import. Moneyball stats come only from the linked FM save file, and metrics that saves do not provide are hidden. The ten pressing roles are scored from their remaining metrics, and role-fit breakdowns mark the missing ones as not available from the save.
- The player shortlist CSV upload, replaced by linked FM shortlist files. Previously imported CSV shortlist entries are deleted on upgrade.
- Moneyball data is kept only on each save's current snapshot. Older snapshots' Moneyball data is deleted on upgrade and whenever a newer snapshot becomes current.
- Moneyball's Potential tactic columns, which matched the Current scores. Moneyball's tactic buttons are now **Add Tactic (Roles)** and **Add Tactic (Custom)**.

### Fixed

- The Position filter now matches players with real familiarity (16 or more) at that position, and Moneyball's Position filter accepts the same position choices as General Search.
- Saving a tactic now refreshes Squad's suggested training and Search's tactic columns.
- The navigation bar no longer clips Dashboard and Settings under display scaling.

## [0.22.0] - 2026-11-07

### Changed

- FM ValueScout now supports Football Manager 27 (build 27.0) instead of FM26. Load Data, player boosts, and staff boosts run against FM27, and the bridge plugin installs into the Steam `Football Manager 27` folder.
- Local graphics detection now looks for the `Football Manager 2027` user-data folder. Choose the FM27 graphics folder again in Settings if you previously chose an FM26 one.
- Updated compatible Rust and frontend dependencies.

### Removed

- FM26 support. The bridge plugin no longer loads data from, or boosts, an FM26 save. Existing FM26 snapshots remain available in the app.

## [0.21.0] - 2026-10-10

### Added

- Workspace loading now keeps the application shell visible and provides retryable error recovery.

### Changed

- Adopted the Signal visual identity with a Night Slate palette, Signal Green accent, bundled Archivo and JetBrains Mono fonts, and new application icons.
- Refined navigation, tables, profiles, Squad, Planner, Tactic, Youth, and Settings with compact framing, clearer grouping, and quieter selection styling.
- Managed-club changes now use an explicit edit, save, and cancel flow.
- Updated compatible Rust dependencies.

### Fixed

- Player and staff shortlist filters now retain each save's session choice across page visits.
- Moneyball evidence and Tactic controls remain side by side at constrained desktop widths, with readable metric details and non-overlapping pitch targets.
- Keyboard focus now enters column menus and returns from closed dialogs, and Youth roster actions remain fully visible when focused.
- Dialogs now dim the workspace with a visible scrim while preserving reduced-motion behavior.
- Planner assignment and Youth class-creation dialogs prevent dismissal while changes are pending.
- Graphics settings now distinguish a cleared folder, active rebuilding, and completed updates.
- Corrected selected pitch-marker and snapshot-label text contrast.

## [0.20.0] - 2026-09-12

### Added

- Staff Search now provides guided assignment optimization with clear setup requirements, compact staffing controls, grouped recommendations, and actionable vacancy details.

### Changed

- Youth-team staff recommendations now balance each candidate's role score with Working With Youngsters, while other assignments keep their existing scoring.

## [0.19.0] - 2026-09-11

### Added

- Settings can now index one local FM graphics root and show pack-supplied player portraits and club logos across Player Profiles, Search, Squad, and My Club. Large packs rebuild in the background, and images load only when needed.
- Settings now provides one confirmed managed-club action that applies Wonderkid Mentality, eligible player CA, and staff CA boosts from one **Load Data** scan.

### Changed

- Suggested Training now limits each developing player's recommended focus to the Physical, Technical, or Mental category for their age.
- **Load Data** no longer retains obsolete accepted-player cap behavior. New scans process all supported players and avoid redundant dump validation and snapshot cleanup while retaining atomic publication and history.
- The FM bridge now uses lower-overhead memory reads and dump writes and checks for **Load Data** requests more often.
- Player boosts now reconcile derived data only for the edited player instead of validating unrelated snapshot rows.
- New **Load Data** scans store exact club identities for graphics. Existing snapshots remain readable but require a fresh scan before club logos can appear.

### Fixed

- Local graphics now refresh when background indexing finishes, load through the packaged Windows protocol, discover both `config.xml` and `_config.xml`, support `r-<uid>` newgen portraits, and preserve fallback club identities for logos.

## [0.18.1] - 2026-09-08

### Added

- Maintainers can now capture populated application routes through deterministic browser fixtures for UI inspection.

### Fixed

- Player shortlist imports now accept FM26's `Unique ID` column, so valid Football Manager exports no longer fail format validation.

## [0.18.0] - 2026-09-08

### Added

- Tactic now provides one phase-aware canvas with persistent Tactical XI and Selected Slot controls, phase-specific occupied-position swaps, responsive portrait and landscape layouts, and clearer IP/OOP markers.
- Planner now shows every enabled squad in one squad-depth board and lets each squad configure, rename, and reorder its strings through Manage Teams.
- Player, Staff, and Squad analysis tables now group related columns, keep identity visible while scrolling, provide grouped column controls, and include Height in fresh player table layouts.
- Player Profiles now provide Overview, Attributes, Role Fit, and Moneyball sections beside a persistent identity rail, with clearer ability and tactical summaries.

### Changed

- Existing custom tactics reset once to the normal defaults for the new tactic layout. Planner assignments and unrelated save data remain intact.
- Existing Planner strings receive ordinal names during upgrade.
- Updated desktop runtime dependencies to their latest compatible patch releases.

### Fixed

- Tactic, Planner, analysis table, and Player Profile layouts now use space more effectively and remain readable across supported desktop widths.

## [0.17.0] - 2026-09-05

### Added

- Player and Staff Search now integrate save-owned shortlist workflows, including CSV imports, shortlist filtering, and staff assignment recommendations.
- Grouped top navigation provides direct access to Home, player and staff search, My Club workspaces including Youth, and Settings.
- Snapshot history can edit an imported snapshot's in-game date.
- Squad shows a Suggested Training focus for developing players, using explicit Planner assignments first and each unassigned player's best-fit tactic lane as a fallback.

### Changed

- General Player Search shortlist filtering now remains independent of Moneyball cohort membership.
- The top utility bar now shows the app logo, a wider global search, and the active snapshot's in-game date while keeping save and data-loading actions together.

### Removed

- Removed the standalone Player Shortlist destination, the My Club Staff Shortlist workspace, the persistent left navigation rail, redundant workspace tabs, frontend player-cap controls, and the snapshot freshness indicator.

## [0.16.0] - 2026-09-03

### Added

- Player Search General, Moneyball, and Shortlist tables now provide optional Current and Potential tactic-lane columns with stable sorting and saved table layouts.
- General scoring now includes the 11 missing FM26 generic out-of-possession roles across Search, player profiles, and Planner. All 88 Moneyball roles now map to General scores.

### Changed

- Channel Midfielder now supports both attacking and central midfield positions.
- Existing score-model version 1 data remains intact but requires a normal **Load Data** run before the expanded version 2 role scores become available.

## [0.15.0] - 2026-09-02

### Added

- Player Search now provides a Shortlist tab for the current Moneyball cohort, with General attributes, filters, sorting, profile analysis, and an independent table layout.
- **Load Data** now reports Scan, Preparing, Scoring, Saving, and Finalizing progress, with phase counts and detailed timing after completion.

### Changed

- Current player and staff role metrics now use compact storage in a fresh `app-v2.db`. The app leaves the legacy `app.db` untouched; verify the new database before manually deleting the old file.

### Fixed

- CSV imports now accept Football Manager exports up to 8 MiB and 10,000 rows.

## [0.14.3] - 2026-08-30

### Fixed

- Position displays now use a consistent canonical order across player, Planner, Search, Squad, Academy, and Moneyball views.
- Academy player lists now show only playable positions with familiarity of 16 or higher, ordered by familiarity and canonical position order for ties.

## [0.14.2] - 2026-08-29

### Fixed

- Staff optimization no longer closes the app when assigning General Coaches in packaged builds.
- Planner optimization now retains automatic lane assignments in packaged builds.

## [0.14.1] - 2026-08-29

### Changed

- Staff assignment slots now follow the FM26 Coaching, Recruitment, and Medical catalog, show club-wide roles once, support Recruitment Analysts, and retain accepted optimizer results when collapsed.
- The optimizer reserves candidates for Head and Chief roles before ordinary slots and matches General Coaches to their required coaching specializations.
- Existing assignment slot counts reset once to adopt the FM26 catalog, while legacy Set Piece Coach targets consolidate into one Club slot.

### Fixed

- Coaches slots now require the correct General, Goalkeeping, and Fitness composition, preserving typed vacancies when a required coach type is unavailable.
- Configure Slots and optimizer results now share canonical squad and role ordering, with every Head role before its ordinary counterpart.

## [0.14.0] - 2026-08-26

### Added

- Squad now provides the shared Moneyball CSV import, and repeated imports accumulate data without removing omitted players.
- Staff Shortlist can define team-specific staffing targets and optimize assignments by job.

### Changed

- Player Search and Squad keep current rows visible during sorting and use indexed ordering for more responsive large-save tables.
- Current-snapshot potential attributes and role scores are materialized once and reused across Search, Squad, Planner, and player profiles.

### Fixed

- Navigation no longer runs repeated snapshot-wide potential-score validation for players and roles unrelated to the requested view.

## [0.13.0] - 2026-08-24

### Added

- My Club now provides one save-owned Club DNA definition with configurable player attributes.
- Player Search and Squad can display and sort Club DNA scores, and Player Search can filter them.

### Changed

- Back and Forward controls now sit before global search in the app header.
- Player profiles now use accessible nationality flags, compact identity and action layouts, and omit the unsupported sweeper position.
- Potential attributes and role scores now match current values for players aged 29 and older.
- Red-tier position familiarity values use muted surfaces while preserving readable and selected-state contrast.

## [0.12.0] - 2026-08-23

### Added

- The app header now provides Back and Forward controls for the current session's route history.
- Moneyball Player Profiles now show their natural-position comparison basis and clearly mark unavailable scores.

### Changed

- Managed-club search and save controls share one responsive row.
- Shared player tables now deduplicate secondary nationalities and stack player identity above club and division; saved layouts migrate duplicate identity columns.
- Moneyball Player Profile scoring now uses natural-position cohorts, and General and Moneyball headers keep stable geometry.

### Fixed

- Staff and Staff Shortlist scrolling remains contained in My Club workspaces.

## [0.11.1] - 2026-08-22

### Fixed

- Managed-club selection no longer restores an outdated club after the active save or snapshot context refreshes.

## [0.11.0] - 2026-08-21

### Added

- Planner now provides a Best role fit reference that independently ranks managed-club players for every IP and OOP tactic role, with current and potential scores.
- The Best role fit reference can switch tactic phases and score basis and sort players by role score.

## [0.10.0] - 2026-08-20

### Added

- Moneyball now provides 88 versioned, position-family-specific role scores from imported performance percentiles.
- Moneyball Player Profiles show best IP/OOP role summaries, a position-filtered role-fit panel, and score explanations.
- Moneyball Player Search adds optional role-score columns, numeric filters, and sorting for Full CSV and filtered comparison cohorts.

## [0.9.0] - 2026-08-20

### Added

- Moneyball CSV imports now store a current-snapshot player cohort with computed metrics and whole-cohort percentile scores.
- Player Search now provides an optional Moneyball view with cohort-only results, metric filters, configurable columns, and virtualized results.
- Player Profiles now provide a Moneyball analysis view, and Settings can make it the default analysis view.

## [0.8.0] - 2026-08-19

### Added

- My Club now provides URL-backed Squad, Planner, Tactic, Staff, and Staff Shortlist workspaces for the selected managed club.

### Changed

- Managed-club selection now lives in My Club, while Settings remains focused on save, snapshot, and bridge management.
- Player Search and Staff Search are explicit standalone destinations, and legacy Planner and Staff workspace links redirect to their canonical My Club URLs.

## [0.7.0] - 2026-08-18

### Added

- Settings now manages saves, snapshots, the FM bridge, and one save-scoped managed club selected from the latest snapshot.

### Changed

- Squad, Planner, Academy, My Staff, and club-wide boosts now derive membership from exact managed-club matches in the latest snapshot.
- Planner teams now share the complete managed-club player pool while preserving age limits, team priority, and save-wide uniqueness.
- Dashboard is now a placeholder while active-save selection and **Load Data** remain in the top bar.

### Removed

- Removed manual Senior, Reserves, and Youth club-source configuration, the Dashboard CSV importer, and the Dashboard sanity-player list.

## [0.6.4] - 2026-08-18

### Fixed

- Planner optimization now accepts familiarity of 12 in both tactic phases and applies hidden ranking deductions below 16.

## [0.6.3] - 2026-08-18

### Fixed

- Planner teams can now be configured per save, with editable display names and safe removal of unneeded teams.

## [0.6.2] - 2026-08-18

### Fixed

- Planner optimizer eligibility now requires familiarity of 16 for the IP position and accepts 12 for a distinct OOP position.

## [0.6.1] - 2026-08-18

### Fixed

- Manager job-fit scores now use Motivating, People Management, Judging Player Ability, Judging Player Potential, and Tactical Knowledge. Staff Shortlists now display and rank a Manager preferred job by that score.

## [0.6.0] - 2026-08-17

### Added

- Staff Shortlist imports a save-owned Football Manager staff CSV, keeps Preferred Job, Club Job, and Coaching Qualifications, and shows only current matched staff.
- Staff Shortlist filters exact Preferred Job values and unemployed staff, and adapts role-score columns for direct jobs and Coach.

## [0.5.2] - 2026-08-17

### Fixed

- Tactic positions can use distinct right, centre, and left central placements without losing role compatibility or optimizer familiarity.

## [0.5.1] - 2026-08-17

### Fixed

- Scrollable suggestion menus remain visible and interactive when opened inside clipped panels, modal scroll regions, or the app shell.

## [0.5.0] - 2026-08-16

### Added

- A Staff workspace now provides full-snapshot Staff Search with configurable columns, filters, sorting, and 20 current job-fit scores.
- My Staff shows the complete configured Senior, Reserves, and Youth club family and can apply the guarded fixed +10 CA action across eligible staff.
- Staff profiles now show current Coaching, Mental, and Knowledge attributes, ranked job fit, the shared hidden-information preference, and an individual fixed +10 CA action.

### Changed

- Staff and player role scores use the shared accessible four-tier score treatment, and staff attributes match the Player Profile presentation.
- Load Data schema v8 persists the staff attributes and job-fit scores required by the Staff workspace while older snapshots remain readable.

### Fixed

- Working With Youngsters now preserves its raw 1–20 FM value instead of applying reputation scaling.
- Staff data refreshes after save or snapshot context changes, and partial My Staff boost failures retain truthful progress and applied-change feedback.

## [0.4.0] - 2026-08-16

### Added

- Player profiles now display complete FM position familiarity, including sweeper familiarity, from exact scanned values.

### Changed

- Search, Squad, Academy, Planner, optimizer, and role projection now apply explicit recorded, playable, and natural-position thresholds so position labels, best-position selection, and eligibility remain consistent with complete familiarity data.
- New **Load Data** scans use the complete schema-v7 position map while existing schema-v6 snapshots remain readable.

## [0.3.0] - 2026-08-16

### Added

- Player profiles can reveal or conceal potential and hidden information for the active save. The saved preference defaults to revealed and applies to every player in that save.
- Player profiles now use Outfield, Goalkeeping, Hidden, and Personality attribute tabs with goalkeeper-specific defaults and grouping.
- Player summaries now show the best Current IP, Current OOP, Potential IP, and Potential OOP roles.
- The desktop app and Windows bundle now use the FM ValueScout icon assets.

### Changed

- **Load Data** outcome banners can be dismissed and now clear when their originating save context is no longer active.

### Fixed

- Player profile controls and panels remain layout-stable when information is concealed or a **Load Data** outcome reduces the available workspace height.

## [0.2.0] - 2026-08-15

### Added

- **Boost all CA** and **Make all Wonderkids** now show determinate `processed / total` progress in the confirmation modal.
- Squad overview feedback now keeps the final boost result stable with updated, skipped, and failed counts, and explains when **Load Data** is required before another action.

## [0.1.0-alpha.1] - 2026-08-14

### Added

- Initial Windows super-early-alpha release with a local FM26 BepInEx bridge, snapshot-backed player search and profiles, Squad, Planner, Academy, configurable tables, CSV enrichment, and guarded player-boost actions.
- A bundled bridge build, checksum, bounded local diagnostics, and a manual recovery path for the unsigned Windows installer.
- Automatic prerelease publication only after a successful required `Check` push run on `main`, with the exact dated section as its GitHub release notes.

### Fixed

- Windows release validation now preserves the prepared changelog notes across line-ending conventions.
- Release publication now preserves absent GitHub SHAs as null instead of treating them as orphaned tags.

### Security

- Local-only application data, with no accounts, cloud sync, telemetry, or automatic data upload.
