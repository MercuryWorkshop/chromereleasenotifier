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
	print("output of the current release does not exist, creating")
	curhash = subprocess.check_output(["/home/e/chromereleasenotifier/target/release/chromereleasesnotifier", "print"]).decode('utf8');
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
		curhash = subprocess.check_output(["/home/e/chromereleasenotifier/target/release/chromereleasesnotifier", "print"]).decode('utf8')
		oldhash = chashfile.read();
		if oldhash != curhash:
			print("chrome release detected!")
			channel = bot.get_channel(channelId)
			await channel.send("<@&1134322964448432138> konqi has delivered", embed=createEmbed(curhash, oldhash))
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

def createEmbed(curout,oldout):
	if not curout:
	    crnprint = subprocess.check_output(["/home/e/chromereleasenotifier/target/release/chromereleasesnotifier", "print"]).decode('utf8')
	else: 
	    crnprint = curout
	crnprint = crnprint.split(sep="__CUT_HERE")
	if oldout:
	    oldoutsplit = oldout.split(sep="__CUT_HERE")
	    crnprint = [x for x in crnprint if x not in oldoutsplit] 

	embed=nextcord.Embed(title="Chrome Releases Notifier", description="Releases fetched from Google", url="https://chromereleases.googleblog.com/")
	embed.set_author(name=random.choice(people), url="https://mercurywork.shop", icon_url="https://cdn.discordapp.com/attachments/1040039623323299992/1095547450019827773/0755fbdafab457bc.png")
	embed.set_thumbnail(url="https://media.discordapp.net/attachments/1040039623323299992/1095547290845970462/250px-Mascot_konqi.png")
	embed.set_footer(text="Delivered proudly by Konqi :3")

	for p in crnprint:
		embed.add_field(name=("New Chrome Release" if oldout else "Chrome Release"), value=p, inline=False)
	return embed

timedfetch.start()
bot.run(TOKEN)
