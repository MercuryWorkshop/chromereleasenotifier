import nextcord
from nextcord.ext import commands, tasks
import subprocess
import random
from hashlib import md5
import os

SCRDIR = os.path.dirname(os.path.abspath( __file__ ))
TOKEN="kde plasma token"
people = ["CoolElectronics", "Rafflesia", "r58Playz", "kotlin", "Astral", "Catakang", "avd3"]
channelId=1066136075703177337
if not (os.path.exists(f"{SCRDIR}/prevoutput.out")):
	print("md5 of the current release does not exist, creating")
	curhash = md5(subprocess.check_output(["/home/e/chromereleasenotifier/target/release/chromereleasesnotifier", "print"])).hexdigest()
	with open(f"{SCRDIR}/prevoutput.out", "w") as file:
		file.write(curhash)
		file.close()

bot = commands.Bot()

@bot.event
async def on_ready():
	print("bot up")

@bot.slash_command()
async def fetchreleases(ctx):
	await ctx.send(embed=createEmbed())

@tasks.loop(minutes=5)
async def timedfetch():
	with open(f"{SCRDIR}/prevoutput.out", "r") as chashfile:
		curhash = md5(subprocess.check_output(["/home/e/chromereleasenotifier/target/release/chromereleasesnotifier", "print"])).hexdigest()
		if chashfile.read() != curhash:
			print("chrome release detected!")
			channel = bot.get_channel(channelId)
			await channel.send("<@&1134322964448432138> konqi has delivered", embed=createEmbed())
			print("sent timed message")
			chashfile.close()
			with open(f"{SCRDIR}/prevoutput.out", "w") as chashfilew:
			    chashfilew.truncate()
			    chashfilew.write(curhash)
			    chashfilew.close()
		else:
			print("no new chrome release")



@timedfetch.before_loop
async def beforetimedfetch():
	await bot.wait_until_ready()

def createEmbed():
	crnprint = subprocess.check_output(["/home/e/chromereleasenotifier/target/release/chromereleasesnotifier", "print"]).decode('utf8')
	crnprint = crnprint.split(sep="__CUT_HERE")

	crnprint.pop()

	embed=nextcord.Embed(title="Chrome Releases Notifier", description="Releases fetched from Google", url="https://chromereleases.googleblog.com/")
	embed.set_author(name=random.choice(people), url="https://mercurywork.shop", icon_url="https://cdn.discordapp.com/attachments/1040039623323299992/1095547450019827773/0755fbdafab457bc.png")
	embed.set_thumbnail(url="https://media.discordapp.net/attachments/1040039623323299992/1095547290845970462/250px-Mascot_konqi.png")
	embed.set_footer(text="Chrome Releases Notifier")

	for p in crnprint:
		embed.add_field(name="Chrome Release", value=p, inline=False)
	return embed

timedfetch.start()
bot.run(TOKEN)
