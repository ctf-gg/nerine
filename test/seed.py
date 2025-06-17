import os
from math import ceil, log

print("THIS SCRIPT IS DESTRUCTIVE!! IT WILL REMOVE ANY DATA IN THE DATABASE")
if input("Type y to confirm, anything else to cancel: ").lower() != "y":
    print("Aborted")
    exit(0)

CHALL_COUNT = 70
CATA_COUNT = 4
TEAM_COUNT = 1000
# on every tick each team as a n% chance
# of submissing a flag, where n is the team's skill
TICKS = 30
SKILL_MEAN = 0.05
SKILL_STD = 0.02
# chance of submitting an incorrect flag
MISS_CHANCE = 0.3

# UTIL STARTS HERE

import random

FRUIT = ["apple", "apricot", "avocado", "banana", "berry", "cantaloupe", "cherry", "citron", "citrus", "coconut", "date", "fig", "grape", "guava", "kiwi", "lemon", "lime", "mango", "melon", "mulberry", "nectarine", "orange", "papaya", "peach", "pear", "pineapple", "plum", "prune", "raisin", "raspberry", "tangerine"]
PLANTS = [ "acorn", "alfalfa", "bamboo", "bark", "bean", "berry", "blade", "brush", "bud", "bulb", "bush", "cactus", "clover", "cork", "corolla", "fern", "flora", "flower", "forest", "garden", "grain", "grass", "grove", "herb", "ivy", "jungle", "juniper", "kelp", "kudzu", "leaf", "lily", "moss", "nectar", "nut", "palm", "petal", "pollen", "resin", "root", "sage", "sap", "seed", "shoot", "shrub", "spore", "stalk", "spine", "sprout", "stem", "thorn", "tree", "trunk", "twig", "vein", "vine", "weed", "wood"]

# adapted from https://github.com/puyuan/py-nanoid
def nanoid(size = 21):
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




def generate_name(prefix = None):
    return f"{f'[{prefix}] 'if prefix else ""}{random.choice(PLANTS)}-{random.choice(FRUIT)}-{random.randint(100, 999)}"

# SEEDING STARTS HERE

import psycopg2
from dotenv import load_dotenv
from psycopg2.extras import execute_batch

load_dotenv()

conn = psycopg2.connect(os.getenv("DATABASE_URL"))

cur = conn.cursor()

cur.execute("""TRUNCATE TABLE 
            teams, 
            categories, 
            challenge_groups, 
            challenges, 
            submissions RESTART IDENTITY""")

categories = []
for i in range(CATA_COUNT):
    categories.append((f"category-{i}",))

print(f"[+] Creating {CATA_COUNT} categories")
execute_batch(cur, "INSERT INTO categories (name) VALUES (%s)", categories)
conn.commit()


challs = []
for i in range(CHALL_COUNT):
    challs.append((nanoid(), generate_name(), "flag", (i % CATA_COUNT) + 1))

cur.execute("SELECT * FROM categories")
print(f"[+] Creating {CHALL_COUNT} challenges")
execute_batch(cur,"""INSERT INTO challenges (public_id, name, author, points_min, points_max, flag, attachments, visible, category_id) 
              VALUES (%s, %s, 'seed', 100, 500, %s, '[]', true, %s)""", challs)
conn.commit()


teams = []
team_skills = []
for i in range(TEAM_COUNT):
    skill = random.gauss(SKILL_MEAN, SKILL_STD)
    name = generate_name(f"skill {skill}")
    team_skills.append(skill)
    teams.append((nanoid(), name, f"{name}@nerine.email"))

print(f"[+] Creating {TEAM_COUNT} teams")
execute_batch(cur,"""INSERT INTO teams (public_id, name, email) 
              VALUES (%s, %s, %s)""", teams)
conn.commit()


from tqdm import tqdm

chall_set = set(range(1, 1 + CHALL_COUNT))
solved_sets = []
for i in range(TEAM_COUNT):
    solved_sets.append(set())

def get_new_chall_id(team_id):
    remaining_challs = chall_set.difference(solved_sets[team_id])
    if len(remaining_challs) == 0: return None
    return random.choice(list(remaining_challs))


for i in tqdm(range(TICKS)):
    submissions = []
    for tid, skill in enumerate(team_skills):
        if random.random() < skill:
            challenge_id = get_new_chall_id(tid)
            if challenge_id == None: continue # team fced
            is_correct = random.random() > MISS_CHANCE
            submissions.append(('yippier!', is_correct, tid + 1, challenge_id, i * 1000))
            if is_correct: solved_sets[tid].add(challenge_id)
    
    execute_batch(cur, """INSERT INTO submissions (submission, is_correct, team_id, challenge_id, created_at) VALUES 
                  (%s, %s, %s, %s,  to_timestamp(%s))""", submissions)
conn.commit()

cur.execute("""WITH solves as (SELECT challenge_id, count(*) AS solves FROM submissions WHERE is_correct = true GROUP BY challenge_id) 
            SELECT c.id, points_min, points_max, solves FROM challenges c JOIN solves ON c.id = challenge_id""")

solves = cur.fetchall()

def point_formula(points_min, points_max, solves):
    return max(points_min, points_max - (points_max - points_min) * solves / 20)


pts_per_chall = map(lambda x: (x[3], point_formula(*x[1:]), x[0]), solves)
for chall in pts_per_chall:
    cur.execute("UPDATE challenges SET c_solves = %s, c_points = %s WHERE id = %s", chall)
conn.commit()


# let points = point_formula(
#     chall_details.points_min,
#     chall_details.points_max,
#     chall_details.solves,
# );

# sqlx::query!(
#     "UPDATE challenges SET c_solves = $1, c_points = $2",
#     chall_details.solves,
#     points
# )
# .execute(db)
# .await?;




print("[!] Seeding done!")
