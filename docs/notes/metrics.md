## Metrics (and considrations on data structure)

Metrics are the statistical data that we use to analyze players in ValueScout. In the tables below, the metrics are sorted into categories (albeit categories are mostly for UX, not for data). Most categories have 2 "types" - Total, and per 90. Some metrics don't, such as ratios, or numbers where a per 90 simply makes no sense (for example, xG per Shot). The source for the data is listed - Most total values are parsed from the CSV file, and most per 90 metrics are computed from the total (from CSV), and the "Minutes" metric. However, because Football Manager is inconsistent, sometimes it's the other way around, where the total must be computed from the per 90, and sometimes the ratio must also be computed from other metrics, often Attempted vs Completed.

There is also the "INVERTED" column - These metrics will often be used be used to calculate percentiles, and be colour-coded based on how good a player is at a given metric. "INVERTED: true" indicates that in such an instance, a lower number is better, whereas in "INVERTED: false", a higher number is better.

The following metrics are not categorized, but are available at all times, and are often used to compute metrics that are not available in the CSV file.

- Appearances
- Minutes

### 1. Attacking & Finishing

_Efficiency and volume of goal-scoring threats._

| METRIC_NAME                       | TOTAL_SOURCE | PER_90_SOURCE | INVERTED |
| --------------------------------- | ------------ | ------------- | -------- |
| Goals                             | CSV          | COMPUTED      | false    |
| Goals from Outside The Box        | CSV          | COMPUTED      | false    |
| xG (Expected Goals)               | CSV          | COMPUTED      | false    |
| NPxG (Non-Penalty Expected Goals) | CSV          | COMPUTED      | false    |
| xG Overperformance                | CSV          | COMPUTED      | false    |
| xG per Shot                       | CSV          | n/a           | false    |
| Shots                             | CSV          | COMPUTED      | false    |
| Shots from Outside The Box        | COMPUTED     | CSV           | false    |
| Shots on Target                   | CSV          | COMPUTED      | false    |
| Shots on Target Ratio             | COMPUTED     | n/a           | false    |
| Conversion Rate                   | COMPUTED     | n/a           | false    |
| Penalties Taken                   | CSV          | COMPUTED      | false    |
| Penalties Scored                  | CSV          | COMPUTED      | false    |
| Penalties Scored Ratio            | COMPUTED     | n/a           | false    |
| Free Kick Shots                   | CSV          | COMPUTED      | false    |
| Average Minutes per Goal          | COMPUTED     | n/a           | true     |
| Average Minutes per Goal/Assist   | COMPUTED     | n/a           | true     |

### 2. Creativity & Chance Creation

_Generating opportunities and final-third delivery._

| METRIC_NAME                      | TOTAL_SOURCE | PER_90_SOURCE | INVERTED |
| -------------------------------- | ------------ | ------------- | -------- |
| Assists                          | CSV          | COMPUTED      | false    |
| xA (Expected Assists)            | CSV          | COMPUTED      | false    |
| Chances Created                  | COMPUTED     | CSV           | false    |
| Clear Cut Chances Created        | CSV          | COMPUTED      | false    |
| Key Passes                       | CSV          | COMPUTED      | false    |
| Open Play Key Passes             | COMPUTED     | CSV           | false    |
| Crosses Attempted                | CSV          | COMPUTED      | false    |
| Crosses Completed                | CSV          | COMPUTED      | false    |
| Cross Completion Ratio           | COMPUTED     | n/a           | false    |
| Open Play Crosses Attempted      | CSV          | COMPUTED      | false    |
| Open Play Crosses Completed      | CSV          | COMPUTED      | false    |
| Open Play Cross Completion Ratio | COMPUTED     | n/a           | false    |
| Average Minutes per Assist       | COMPUTED     | n/a           | true     |

### 3. Transition & Ball Progression

_Movement of the ball through the thirds and individual ball-carrying._

| METRIC_NAME            | TOTAL_SOURCE | PER_90_SOURCE | INVERTED |
| ---------------------- | ------------ | ------------- | -------- |
| Passes Attempted       | CSV          | COMPUTED      | false    |
| Passes Completed       | CSV          | COMPUTED      | false    |
| Pass Completion Ratio  | COMPUTED     | n/a           | false    |
| Progressive Passes     | CSV          | COMPUTED      | false    |
| Dribbles Made          | CSV          | COMPUTED      | false    |
| Distance Covered       | CSV          | COMPUTED      | false    |
| High Intensity Sprints | COMPUTED     | CSV           | false    |
| Possession Lost        | COMPUTED     | CSV           | true     |

### 4. Defensive Actions

_Ball recovery and stopping the opposition._

| METRIC_NAME             | TOTAL_SOURCE | PER_90_SOURCE | INVERTED |
| ----------------------- | ------------ | ------------- | -------- |
| Tackles Attempted       | CSV          | COMPUTED      | false    |
| Tackles Completed       | CSV          | COMPUTED      | false    |
| Tackle Completion Ratio | COMPUTED     | n/a           | false    |
| Key Tackles             | CSV          | COMPUTED      | false    |
| Interceptions           | CSV          | COMPUTED      | false    |
| Possession Won          | COMPUTED     | CSV           | false    |
| Pressures Attempted     | CSV          | COMPUTED      | false    |
| Pressures Completed     | CSV          | COMPUTED      | false    |
| Pressure Success Ratio  | COMPUTED     | n/a           | false    |
| Blocks                  | CSV          | COMPUTED      | false    |
| Shots Blocked Defending | CSV          | COMPUTED      | false    |
| Clearances              | CSV          | COMPUTED      | false    |

### 5. Aerial Presence

_Dominance in the air during both phases._

| METRIC_NAME                 | TOTAL_SOURCE | PER_90_SOURCE | INVERTED |
| --------------------------- | ------------ | ------------- | -------- |
| Aerial Challenges Attempted | CSV          | COMPUTED      | false    |
| Headers Won                 | CSV          | COMPUTED      | false    |
| Headers Lost                | COMPUTED     | CSV           | true     |
| Headers Won Ratio           | COMPUTED     | n/a           | false    |
| Key Headers                 | COMPUTED     | CSV           | false    |

### 6. Goalkeeping & Shot Stopping

_Preventing goals and handling opposition strikes._

| METRIC_NAME                    | TOTAL_SOURCE | PER_90_SOURCE | INVERTED |
| ------------------------------ | ------------ | ------------- | -------- |
| Clean Sheets                   | CSV          | COMPUTED      | false    |
| Goals Conceded                 | CSV          | COMPUTED      | true     |
| Total Saves                    | COMPUTED     | CSV           | false    |
| Save Ratio                     | COMPUTED     | n/a           | false    |
| xSP (Expected Save Percentage) | CSV          | n/a           | false    |
| xGP (Expected Goals Prevented) | CSV          | COMPUTED      | false    |
| Saves Held                     | CSV          | COMPUTED      | false    |
| Saves Parried                  | CSV          | COMPUTED      | false    |
| Saves Tipped                   | CSV          | COMPUTED      | false    |
| Penalties Faced                | CSV          | COMPUTED      | false    |
| Penalties Saved                | CSV          | COMPUTED      | false    |
| Penalties Saved Ratio          | COMPUTED     | n/a           | false    |

### 7. Discipline & Error Margins

_Negative actions and off-ball mistakes._

| METRIC_NAME              | TOTAL_SOURCE | PER_90_SOURCE | INVERTED |
| ------------------------ | ------------ | ------------- | -------- |
| Fouls Made               | CSV          | COMPUTED      | true     |
| Fouls Against            | CSV          | COMPUTED      | false    |
| Yellow Cards             | CSV          | COMPUTED      | true     |
| Red Cards                | CSV          | COMPUTED      | true     |
| Offsides                 | CSV          | COMPUTED      | true     |
| Mistakes Leading to Goal | CSV          | COMPUTED      | true     |

### 8. Match Impact & Availability

_Team-level contributions and player consistency._

| METRIC_NAME         | TOTAL_SOURCE | PER_90_SOURCE | INVERTED |
| ------------------- | ------------ | ------------- | -------- |
| Average Rating      | CSV          | n/a           | false    |
| Player of the Match | CSV          | n/a           | false    |
| Game Win Ratio      | COMPUTED     | n/a           | false    |
| Games Won           | CSV          | n/a           | false    |
| Games Drawn         | CSV          | n/a           | true     |
| Games Lost          | CSV          | n/a           | true     |
| Team Goals          | CSV          | COMPUTED      | false    |

## Archetypes

This will be a list of the default Archetypes, and the metrics and weight used to calculate them.

### Goalkeeper

#### In Possession

##### Traditional Goalkeeper

- Pass Completion Ratio (0.70)
- Passes Attempted per 90 (0.30)

##### Ball-Playing Goalkeeper

- Progressive Passes (0.40)
- Pass Completion Ratio (0.30)
- Passes Attempted per 90 (0.20)
- Expected Assists (xA) (0.10)

#### Out of Possession

##### Traditional Goalkeeper

- Expected Goals Prevented per 90 (0.50)
- Save Ratio (0.30)
- Saves Held (0.20)

##### Sweeper Keeper

- Expected Goals Prevented per 90 (0.40)
- Interceptions per 90 (0.30)
- High Intensity Sprints per 90 (0.20)
- Distance Covered per 90 (0.10)

### Center Back

#### In Possession

##### Traditional Center Back

- Pass Completion Ratio (0.60)
- Passes Completed per 90 (0.40)

##### Ball-Playing Center Back

- Progressive Passes per 90 (0.45)
- Pass Completion Ratio (0.25)
- Passes Attempted per 90 (0.20)
- Key Passes per 90 (0.10)

#### Out of Possession

##### Center Back

- Headers Won Ratio (0.30)
- Interceptions per 90 (0.25)
- Possession Won per 90 (0.20)
- Blocks per 90 (0.15)
- Clearances per 90 (0.10)

### Right Back / Left Back

#### In Possession

##### Full Back

- Pass Completion Ratio (0.40)
- Passes Attempted per 90 (0.30)
- Progressive Passes per 90 (0.20)
- Distance Covered per 90 (0.10)

##### Offensive Full Back

- Expected Assists per 90 (0.35)
- Open Play Crosses Completed per 90 (0.25)
- Dribbles Made per 90 (0.20)
- Key Passes per 90 (0.20)

#### Out of Possession

##### Pressing Full Back

- Pressures Completed per 90 (0.35)
- Possession Won per 90 (0.25)
- High Intensity Sprints per 90 (0.25)
- Tackles per 90 (0.15)

##### Defensive Full Back

- Tackle Completion Ratio (0.35)
- Interceptions per 90 (0.25)
- Headers Won Ratio (0.20)
- Blocks per 90 (0.20)

### Defensive Midfielder

#### In Possession

##### Defensive Midfielder

- Pass Completion Ratio (0.45)
- Passes Completed per 90 (0.30)
- Progressive Passes per 90 (0.15)
- Possession Lost per 90 (inverted) (0.10)

##### Playmaker

- Progressive Passes per 90 (0.40)
- Expected Assists per 90 (0.20)
- Passes Attempted per 90 (0.20)
- Key Passes per 90 (0.20)

#### Out of Possession

##### Pressing Defensive Midfielder

- Pressures Completed per 90 (0.35)
- Possession Won per 90 (0.30)
- Tackles per 90 (0.20)
- Interceptions per 90 (0.15)

##### Pure Playmaker (no to little defensive responsibility)

- Interceptions per 90 (0.45)
- Distance Covered per 90 (0.25)
- Fouls Made (0.20)
- Mistakes Leading to Goal (inverted) (0.10)

### Right Wingback / Left Wingback

#### In Possession

##### Wing Back

- Crosses Completed per 90 (0.35)
- Passes Attempted per 90 (0.25)
- Progressive Passes per 90 (0.20)
- Distance Covered per 90 (0.20)

##### Offensive Wing Back

- Expected Assists per 90 (0.35)
- Dribbles Made per 90 (0.25)
- Chances Created per 90 (0.20)
- Open Play Crosses Completed per 90 (0.20)

#### Out of Possession

##### Pressing Wing Back

- Pressures Completed per 90 (0.40)
- Possession Won per 90 (0.30)
- High Intensity Sprints per 90 (0.20)
- Tackles Attempted per 90 (0.10)

##### Tracking Wing Back

- Distance Covered per 90 (0.40)
- Interceptions per 90 (0.30)
- Tackle Completion Ratio (0.20)
- Blocks per 90 (0.10)

### Central Midfielder

#### In Possession

##### All-Rounder

- Pass Completion Ratio (0.30)
- Passes Attempted per 90 (0.25)
- Progressive Passes per 90 (0.25)
- Distance Covered per 90 (0.20)

##### Box-to-Box

- Expected Goals per 90 (0.30)
- Distance Covered per 90 (0.25)
- Shots per 90 (0.25)
- Progressive Passes per 90 (0.20)

##### Playmaker

- Expected Assists per 90 (0.35)
- Progressive Passes per 90 (0.30)
- Key Passes per 90 (0.20)
- Pass Completion Ratio (0.15)

#### Out of Possession

##### All-Rounder

- Tackles per 90 (0.30)
- Interceptions per 90 (0.25)
- Pressures Completed per 90 (0.25)
- Possession Won per 90 (0.20)

##### Covering Midfielder

- Interceptions per 90 (0.40)
- Tackle Completion Ratio (0.30)
- Blocks per 90 (0.20)
- Distance Covered per 90 (0.10)

### Left Winger / Right Winger

#### In Possession

##### Traditional Winger

- Crosses Completed per 90 (0.40)
- Dribbles Made per 90 (0.30)
- Key Passes per 90 (0.20)
- Pass Completion Ratio (0.10)

##### Goalscoring Winger

- Non Penalty Expected Goals per 90 (0.45)
- Shots on Target Ratio (0.25)
- Conversion Rate (0.20)
- Goals per 90 (0.10)

##### Inside Forward

- xG per 90 (0.30)
- xA per 90 (0.25)
- Dribbles Made per 90 (0.25)
- Key Passes per 90 (0.20)

#### Out of Possession

##### Offensive Outlet

- High Intensity Sprints per 90 (0.40)
- Expected Goals per 90 (0.30)
- Fouls Against (0.20)
- Offsides (0.10)

##### Tracking Winger

- Distance Covered per 90 (0.35)
- Pressures Completed per 90 (0.30)
- Tackles per 90 (0.20)
- Interceptions per 90 (0.15)

### Attacking Midfielder

#### In Possession

##### Running Attacking Mid

- Non Penalty xG per 90 (0.40)
- Shots per 90 (0.25)
- Distance Covered per 90 (0.20)
- Goals per 90 (0.15)

##### Playmaker

- xA per 90 (0.35)
- Key Passes per 90 (0.30)
- Progressive Passes per 90 (0.20)
- Pass Completion Ratio (0.15)

#### Out of Possession

##### Offensive Outlet

- High Intensity Sprints per 90 (0.40)
- Non Penalty xG per 90 (0.30)
- Fouls Against (0.20)
- Offsides (0.10)

##### Tracking Attacking Mid

- Pressures Completed per 90 (0.40)
- Distance Covered per 90 (0.30)
- Tackles per 90 (0.15)
- Interceptions per 90 (0.15)

### Striker

#### In Possession

##### Creative Forward

- xA per 90 (0.35)
- Key Passes per 90 (0.25)
- Progressive Passes per 90 (0.20)
- Pass Completion Ratio (0.10)
- Dribbles Made per 90 (0.10)

##### Goalscoring Forward

- Non Penalty xG per 90 (0.45)
- Expected Goals per Shot (0.25)
- Shots on Target Ratio (0.15)
- Conversion Rate (0.10)
- Average Minutes per Goal (0.05)

#### Out of Possession

##### Offensive Outlet

- High Intensity Sprints per 90 (0.40)
- Non Penalty xG per 90 (0.30)
- Fouls Against (0.20)
- Offsides (0.10)

##### Pressing Forward

- Pressures Completed per 90 (0.40)
- Possession Won per 90 (0.30)
- Distance Covered per 90 (0.20)
- Tackles per 90 (0.10)
