# Like seed.py, but using more realistic data.
import os
import psycopg2
import random
from psycopg2.extras import execute_batch
from dotenv import load_dotenv
from math import ceil, log
from tqdm import tqdm

# Number of submissions each team attempts
TEAM_SUBMISSIONS = 100
# Submissions per tick
SUBMISSIONS_PER_TICK = 5

# adapted from https://github.com/puyuan/py-nanoid
def nanoid(size=21):
    alphabet = "_-0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    alphabet_len = len(alphabet)

    mask = 1
    if alphabet_len > 1:
        mask = (2 << int(log(alphabet_len - 1) / log(2))) - 1
    step = int(ceil(1.6 * mask * size / alphabet_len))

    id = ''
    while True:
        random_bytes = bytearray(os.urandom(step))

        for i in range(step):
            random_byte = random_bytes[i] & mask
            if random_byte < alphabet_len:
                if alphabet[random_byte]:
                    id += alphabet[random_byte]

                    if len(id) == size:
                        return id

print("THIS SCRIPT IS DESTRUCTIVE!! IT WILL REMOVE ANY DATA IN THE DATABASE")
if input("Type y to confirm, anything else to cancel: ").lower() != "y":
    print("Aborted")
    exit(0)

load_dotenv()

# Top 100 teams on CTFtime, 2025-06-06
top_teams = ["kalmarunionen", "Infobahn", "L3ak", "r3kapig", "valgrind", "thehackerscrew", "The Flat Network Society", "RubiyaLab", "S™", "SNI", "Squid Proxy Lovers", "Maple Bacon", "Cryptonite", "justCatTheFish", ".;,;.", "Srdnlen", "Project Sekai", "TheRomanXpl0it", "noreply", "organizers", "💦​", "QnQSec", "BITSkrieg", "DeadSec", "bi0s", "S1uM4i", "PBR | UCLA", "PwnInfra", "InfoSecIITR", "Never Stop Exploiting", "FPTU Ethical Hacker Club", "> r0/dev/null", "🐧‎", "Tower of Hanoi", "HCS", "Securani", "Ganesh", "b01lers", "MindCrafters", "Marcio Herobrine", "BKISC", "f4n_n3r0", "Ireland without the RE", "FlagMotori", "Gimel", "Nu1L", "0xFUN", "Thread in the Needle", "NYUSEC", "H7Tex", "slight_smile", "Nc{Cat}", "SU", "BlitzHack", "0xL4ugh", "0xHUST", "st0p_cyb3rbu11ying", "STT", "Plaid Parliament of Pwning", "Zer0RocketWrecks", "Lil L3ak", "0-Day Aarhus", "DiceGang", "FluxFingers", "RaptX", "BunkyoWesterns", "0bscuri7y", "Singapore Students Merger", "nahomies", "L.A.R.P.", "saladstream", "dtl", "Superflat", "SaturnX", "PwnSec", "SNHT", "UniverSea", "twosheep", "Shellphish", "UofTCTF", "Cosmic Bit Flip", "0xFAILURES", "KITCTF", "0ops", "deathwing", "Cyb3rPr1s0n", "Hakierspejs", "Akasec", "Bing Chilling Academies", "xSTF", "CascRoot", "idek", "ECSC Slovak Cyber Team", "0bug", "FR13NDS TEAM", "NUS GreyHats", "pwnthem0le", "The power of Elijah", "bingus", "NOVA"]
top_team_points = [698.977, 695.400, 667.035, 625.192, 564.954, 546.537, 536.464, 523.526, 497.228, 497.104, 454.850, 448.463, 437.460, 436.192, 423.152, 416.914, 398.127, 397.601, 391.477, 379.280, 372.761, 365.709, 363.863, 356.781, 356.420, 354.616, 351.290, 348.359, 348.162, 346.280, 340.864, 340.600, 337.169, 336.193, 335.818, 335.092, 328.585, 324.128, 317.477, 313.948, 312.608, 311.773, 306.118, 300.734, 298.277, 287.206, 286.962, 281.605, 281.517, 277.501, 275.894, 275.788, 275.743, 273.266, 271.157, 271.122, 270.635, 267.730, 263.479, 263.418, 258.262, 257.483, 257.397, 257.096, 256.891, 255.705, 252.158, 250.014, 249.258, 248.361, 245.758, 245.387, 244.767, 242.207, 241.854, 239.919, 238.174, 234.747, 232.394, 227.937, 226.951, 225.473, 221.758, 220.706, 217.969, 217.757, 217.486, 214.159, 213.245, 213.102, 212.375, 212.259, 209.163, 207.091, 206.045, 205.755, 200.364, 199.514, 198.360, 198.156]

# PlaidCTF 2025 (point values changed)
challenges = [
    # "easy" - 100 pts
    ("rev", "Sanity Check", 100),
    ("pwn", "The Sundown Vault", 100),
    ("misc", "Sheriff Says", 100),
    ("web", "Plaid Apple!!", 100),
    ("misc", "Prospuccin'", 100),
    ("misc", "innov8", 100),
    # "medium" - 300 pts
    ("web", "ChatPPP", 300),
    ("misc", "Rumbleweed Inc.", 300),
    ("pwn", "Many Mellons", 300),
    ("web", "Yeehaw!", 300),
    ("pwn", "Bounty Board", 300),
    ("crypto", "TaleS from the Crypt", 300),
    # "hard" - 500 pts
    ("pwn", "ocalc", 500),
    ("web", "Trading Post", 500),
    ("pwn", "Zerodeo", 500),
    ("crypto", "hangman", 500),
]
start_ts = 1743825600
end_ts   = 1743998400
solve_end_ts = start_ts + round((end_ts - start_ts) * 0.95)

categories = list(set(c[0] for c in challenges))
categories.sort()

def email_from_team_name(name: str) -> str:
    inner = "".join(c for c in name.lower() if c in "abcdefghijklmnopqrstuvwxyz0123456789")
    if not inner:
        inner = f"team_{hash(name)}"
    return f"{inner[:54]}@sctf.email"

def flag_from_chall_name(name: str) -> str:
    inner = "".join(c for c in name.lower() if c in "abcdefghijklmnopqrstuvwxyz0123456789 ").replace(" ","_")
    return "plaid{" + inner + "}"

def incorrect_flag() -> str:
    chars = "".join(random.choice("abcdef0123456789") for _ in range(16))
    return f"plaid{{incorrect_{chars}}}"

# Plausibly estimate how many challenges each team would have solved
min_team_points = min(top_team_points)
max_team_points = max(top_team_points)
top_team_solves = [max(1, round((p - min_team_points) / (max_team_points - min_team_points) * len(challenges))) for p in top_team_points]
print(top_team_solves)

### Seeding part

conn = psycopg2.connect(os.environ["DATABASE_URL"])
cur = conn.cursor()

cur.execute("""
TRUNCATE TABLE teams, categories, challenge_groups, challenges, submissions RESTART IDENTITY
""")

print(f"[+] Creating {len(categories)} categories: {", ".join(categories)}")
execute_batch(cur, "INSERT INTO categories (name) VALUES (%s)", [(x,) for x in categories])
conn.commit()

chall_rows = [(nanoid(), c[1], "seed", 20, c[2], flag_from_chall_name(c[1]), "[]", True, categories.index(c[0])+1) for c in challenges]
print(f"[+] Creating {len(challenges)} challenges")
execute_batch(cur, """
INSERT INTO challenges (public_id, name, author, points_min, points_max, flag, attachments, visible, category_id)
VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
""", chall_rows)
conn.commit()

team_rows = [(nanoid(), name, email_from_team_name(name)) for name in top_teams]
print(f"[+] Creating {len(top_teams)} teams")
execute_batch(cur, """
INSERT INTO teams (public_id, name, email)
VALUES (%s, %s, %s)
""", team_rows)

team_submissions = []
team_solves = {j: 0 for j in range(len(top_teams))}

num_ticks = len(top_teams)*TEAM_SUBMISSIONS / SUBMISSIONS_PER_TICK
if not num_ticks.is_integer():
    print(f"[*] warning: non-integer number of ticks {num_ticks}, will be rounded")
num_ticks = round(num_ticks)
team_submissions_per_tick = round(TEAM_SUBMISSIONS / num_ticks)

print(f"[+] Simulating {num_ticks} ticks...")
actual_submissions = 0
tick_iter = tqdm(range(num_ticks))
for i in tick_iter:
    target_updates = {}
    ts = round(start_ts + ((i+1) / num_ticks) * (solve_end_ts - start_ts))
    remaining_ticks = num_ticks - (i+1)
    submissions_this_tick = SUBMISSIONS_PER_TICK
    tick_submissions = []
    random_submissions_this_tick = {j: 0 for j in range(len(top_teams))}

    for j, team in enumerate(top_teams):
        expected_solves = top_team_solves[j]
        # approximate
        #remaining_submissions_this_team = int(remaining_ticks * SUBMISSIONS_PER_TICK / len(top_teams))
        remaining_submissions_this_team = remaining_ticks * SUBMISSIONS_PER_TICK / len(top_teams)
        solves_so_far = team_solves[j]
        # when remaining_ticks is 0, this will always be exact
        if remaining_ticks == 0 or remaining_submissions_this_team == 0:
            needed_solves_this_tick = expected_solves - solves_so_far
        else:
            needed_solves_this_tick = (expected_solves - solves_so_far) / remaining_submissions_this_team / remaining_ticks

        if needed_solves_this_tick > 0 and len(tick_submissions) + needed_solves_this_tick > SUBMISSIONS_PER_TICK:
            tick_iter.write(f"[-] WARNING: tick {i} requires {needed_solves_this_tick} for team {team} ({j}) submissions which exceeds {SUBMISSIONS_PER_TICK} ({len(tick_submissions)} existing)")
            #exit(1)

        needed_solves_this_tick_i = int(needed_solves_this_tick)
        needed_solves_this_tick_f = needed_solves_this_tick - needed_solves_this_tick_i
        if random.random() < needed_solves_this_tick_f:
            needed_solves_this_tick_i += 1
        challs_solved_this_tick = challenges[solves_so_far:solves_so_far+needed_solves_this_tick_i]
        #submissions_this_tick = [(flag_from_chall_name(c[1]), True, j+1, solves_so_far+k+1) for k, c in enumerate(challs_solved_this_tick)]
        tick_submissions.extend((flag_from_chall_name(c[1]), True, j+1, solves_so_far+k+1) for k, c in enumerate(challs_solved_this_tick))
        #tick_submissions.extend((*l, ts) for l in submissions_this_tick)
        team_solves[j] += needed_solves_this_tick_i

    # perform random submissions
    #for j, random_submissions in random_submissions_this_tick.items():
    random_submissions_left = SUBMISSIONS_PER_TICK - len(tick_submissions)
    tick_iter.write(f"[*] Performing {random_submissions_left} random submissions on tick {i}")
    if random_submissions_left > 0:
        succeed_probability = 1/(remaining_ticks+1)
        fc_visited = set()
        while random_submissions_left > 0:
            j = random.choice(range(len(top_teams)))
            remaining_solves = top_team_solves[j] - team_solves[j]
            if remaining_solves == 0:
                fc_visited.add(j)
                if len(fc_visited) == len(top_teams):
                    break
                # already cleared
                continue
            if random.random() < succeed_probability:
                # pick the next solve
                c = challenges[team_solves[j]]
                tick_submissions.append((flag_from_chall_name(c[1]), True, j+1, team_solves[j]+1))
                team_solves[j] += 1
            else:
                tick_submissions.append((incorrect_flag(), False, j+1, random.choice(range(len(challenges)))+1))
            random_submissions_left -= 1

    actual_submissions += len(tick_submissions)
    team_submissions.extend((*l, ts) for l in tick_submissions)

print(f"[*] Performed {actual_submissions} actual submissions")

execute_batch(cur, """
INSERT INTO submissions (submission, is_correct, team_id, challenge_id, created_at)
VALUES (%s, %s, %s, %s, to_timestamp(%s))
""", team_submissions)

print(f"[+] Updating challenge points...")

cur.execute("""
WITH solves as (SELECT challenge_id, count(*) AS solves FROM submissions WHERE is_correct = true GROUP BY challenge_id)
SELECT c.id, points_min, points_max, solves FROM challenges c JOIN solves ON c.id = challenge_id
""")

solves = cur.fetchall()

def point_formula(points_min, points_max, solves):
    return max(points_min, points_max - (points_max - points_min) * solves / 20)

pts_per_chall = map(lambda x: (x[3], point_formula(*x[1:]), x[0]), solves)
for chall in pts_per_chall:
    cur.execute("UPDATE challenges SET c_solves = %s, c_points = %s WHERE id = %s", chall)
conn.commit()

print("[+] Done!")
