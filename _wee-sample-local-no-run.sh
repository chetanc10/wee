#!/bin/bash

# If you need new wee local script for your work/sdk environment:
# 1. make a copy of _wee-sample-local-no-run.sh
# 2. rename it as wee-<custom-name>-local.sh
# 3. NO_MODIFY blocks MUST NEVER BE modified
# 4. DO_MODIFY blocks CAN BE modified/removed to suit dev-environment
# 5. edit variable/function definitions per local server configuration
# 6. each wee function must support '-h' option for help msg
# 7. wee function with '_' prefix is wee-internal, MUST NOT be used directly

# NO_MODIFY
# Ensure that we are sourced by weemain.sh only
([ -z "${wee}" ] || [ ! -d "${wee}" ]) && \
	echo2 "ERROR: $BASH_SOURCE MUST be exported by weemain.sh" && \
	(return 0 2>/dev/null) && return 0
([ -z "${wee}" ] || [ ! -d "${wee}" ]) && exit 0


###########################################################
########### Helpers to copy to/from NW folders ############
###########################################################

# DO_MODIFY
# Network Path used to share/access data among systems/users
export NWP="/smb/nw4-neptune/bermuda/my_dir"

# NO_MODIFY
# Copy local files to NW folder
nwo ()
{
	[ -z "${1}" ] && \
		echo "Usage: ${FUNCNAME[0]} <file/folder path>" && \
		return 0
	[ ! -d "${NWP}" ] && echo "${NWP} not found!" && return 1
	cp -rf "${1}" ${NWP}/
	chmod o+r ${NWP}/$(basename "${1}")
	return $?
}

# NO_MODIFY
# Copy local files from NW folder
nwi ()
{
	[ -z "${1}" ] && \
		echo "Usage: ${FUNCNAME[0]} <NW file/folder path>" && \
		return 0
	[ ! -d "${NWP}" ] && echo "${NWP} not found!" && return 1
	cp -rf ${NWP}/"${1}" ./ 
	return $?
}

# DO_MODIFY
# Gerrit SSH server info, if used or leave as-is
export gDom="review.googul.com"
export gPort=29418
export gSSHU="ssh://${gDom}:$gPort"
export gHTTPU="https://${gDom}"


###########################################################
#### User Functions/variables for localized environment ###
###########################################################

# NOTE: All variables defined here are REFERENCE_ONLY.
#       Users update required variable(s) as per sdk needs,
#       adding more variables if needed

# DO_MODIFY
# Variable used by sdkcd() to use gSdkDirs
# Refer sdkcd() in weemain.sh
# Recommended gSdkDirs:
# sdk    : Root directory of SDK
# sdks   : main src directory inside SDK
# sdkb   : build directory inside SDK (created after build)
# sdki   : image directory inside SDK (created after build)
# sdkl   : linux kernel directory inside SDK
declare -A gSdkDirs=( \
	["sdk"]="sdk" \
	["sdks"]="sdk/src/" \
	["sdkb"]="sdk/build/target-*/linux-*generic/" \
	["sdki"]="sdk/bin/target-*/" \
	["sdkl"]="sdk/src/linux-5.4*/" \
)

# DO_MODIFY
# Variable used to validate/choose platform
# with platform codes and platform descriptions
declare -A gKnownPlatforms=( \
	["imx6sll-evb"]="i.MX 6SLL EVB" \
	["imx8-dc"]="i.MX 8 Dashcam" \
)

# DO_MODIFY
# . Dummy aliases to cd to various sdk directory paths
# . Aliases here are mostly the same as key strings in
#   gSdkDirs e.g. sdk, sdkb, etc
# . Skip unnecessary typing with these aliases!
# . Platform specific aliases can also be added
#   or just use existing aliases plus -p <platform>
#   e.g. sdkb -p x86
alias sdk="sdk"
alias sdks="sdk -d sdks"
alias sdkb="sdk -d sdkb"
alias sdki="sdk -d sdki"

# DO_MODIFY
# Quick Gcc Toolchain accessor (QGT)
# Not to be used directly, refer QGTAA in weemain.sh
_qgt ()
{
	# This is example for old OpenWRT SDK; UPDATE this
	abd sdk || return $?

	# Get ToolChain Directory
	local tcdir="$(ls -d ${curd}/sdk/staging_dir/toolchain-*)"
	[ -z "${tcdir}" ] && return 1

	local util=${1}; shift
	${tcdir}/bin/*-${util} "${@}"

	return 0
}

# DO_MODIFY
# _gcommitstub is a stub function expected by weemain
# for local repo sdk frameworks enable to handle the way
# different project gits inside a huge repo are committed
# We may take RGRL from weemain.sh for our use
_jpyc ()
{
	# helper for _gcommitstub
	local frp=$(git remote -v | head -n1 | awk '{print $2}')
	local rp="${frp#*$gPort/}"
	local rp="${rp%'.git'/}"
	echo $(cat ${1} | python3 -c "import sys, json; exec(\"try:\n\tprint(json.load(sys.stdin)['${rp}']['${2}']);\nexcept KeyError:\n\tprint('')\")")
}
_manualprojdesc ()
{
	# helper for _gcommitstub
	echo -e "${@}\nConfirm as\n1. Public\n2. Private" >/dev/stdout
	read -p "Enter any other key to exit. Choice: " idx
	case $idx in
		1) echo "Public"; return 0 ;;
		2) echo "Private"; return 0 ;;
		*) echo ""; return -1 ;;
	esac
}
_gcommitstub ()
{
	# If we're not on a git-repo, just return
	git remote 2>/dev/null || return

	# Get git-hooks for Change-Id generation on commits
	local gitdir=$(git rev-parse --git-dir)
	scp -p -P $gPort $(whoami)@${gDom}:hooks/commit-msg ${gitdir}/hooks/

	local desc=""
	# Public commits MUST NOT be signed
	# Private commits MUST be signed
	# If unclear, do not commit at all
	if [ ! -s ${RGRL} ]; then
		desc=$(_manualprojdesc "${PWD} as Public/Private undetermined!")
		[ -z "${desc}" ] && \
			echo "User couldn't confirm Public/Private" && return 1
	else
		desc="$(_jpyc ${RGRL} description)"
	fi

	local opts="${@}"
	if [[ "${desc}" == *"Public"* ]]; then
		#git config user.email "$(whoami)@opensrc.com"
		git commit $opts
	elif [[ "${desc}" == *"Private"* ]]; then
		#git config user.email "$(whoami)@propermail.com"
		git commit $opts -s
	else
		echo "${PWD} not Public/Private. Value:${desc} !"
		return 2
	fi

	return $?
}

