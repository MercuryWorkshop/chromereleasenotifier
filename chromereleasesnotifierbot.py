import nextcord
from nextcord.ext import commands, tasks
import subprocess
import random

TOKEN="kde plasma token"
people = ["CoolElectronics", "Rafflesia", "r58Playz", "kotlin", "Astral", "Catakang", "avd3"]
channelId=1066136075703177337

bot = commands.Bot()

@bot.event
async def on_ready():
	print("bot up")

@bot.slash_command()
async def fetchreleases(ctx):
	await ctx.send(embed=createEmbed())

@tasks.loop(hours=24)
async def timedfetch():
	channel = bot.get_channel(channelId)
	await channel.send("ping here? idk", embed=createEmbed())
	print("sent timed message")

@timedfetch.before_loop
async def beforetimedfetch():
	await bot.wait_until_ready()

def createEmbed():
	crnfull = subprocess.check_output(["/home/e/chromereleasenotifier/target/release/chromereleasesnotifier", "full"]).decode('utf8')
	crnprint = subprocess.check_output(["/home/e/chromereleasenotifier/target/release/chromereleasesnotifier", "print"]).decode('utf8')
	crnfull = crnfull.split(sep="__CUT_HERE")
	crnprint = crnprint.split(sep="__CUT_HERE")

	crnfull.pop()
	crnprint.pop()

	embed=nextcord.Embed(title="Chrome Releases Notifier", description="Releases fetched from Google", url="https://chromereleases.googleblog.com/")
	embed.set_author(name=random.choice(people), url="https://mercurywork.shop", icon_url="https://cdn.discordapp.com/attachments/1040039623323299992/1095547450019827773/0755fbdafab457bc.png")
	embed.set_thumbnail(url="https://media.discordapp.net/attachments/1040039623323299992/1095547290845970462/250px-Mascot_konqi.png")
	embed.set_footer(text="Chrome Releases Notifier")

	for f, p in zip(crnfull, crnprint):
		embed.add_field(name=p, value=f, inline=False)
	return embed

timedfetch.start()
bot.run(TOKEN)
