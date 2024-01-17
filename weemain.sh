#!/bin/bash

# Export and function for wee for later use
export wee=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)

# Simple function to echo any message to stderr
echo2 ()
{
	echo -e "${@}" >/dev/stderr
}

# If wee is given in shell with no args, it does 'cd ${wee}'
wee ()
{
	if [ -z $1 ]; then
		cd "${wee}"
	elif [ -s "wee-$1-local.sh" ]; then
		source "${wee} $1"
	else
		echo2 "WARNING: No proper wee local available!"
	fi
}

###########################################################
################# Generic wee functions ###################
###########################################################

### All wee functions MUST give '-h' option for help msg
### Help msg option is supported by default
### EXCEPTIONS: wee(), echo2()

abd ()
{
	([ -z "${1}" ] || [ "$1" == "-h" ]) && echo \
		"Usage: ${FUNCNAME[0]} <fname>\n" \
		"Find parent-path of given file/directory up the dir-tree\n" \
		"<fname>: File/Directory name to find it's parent\n" \
		&& return 1

	# We don't localize origd/curd because they're needed
	# by callers to move back/forth between target dirs
	# and curd/origd
	origd=${PWD}; curd=${PWD}

	while [ "$(ls -d ${curd}/${1} 2>/dev/null)" == "" ]; do
		if [ "${curd}" == "/" ]; then
			echo "No file/directory named ${1} up the directory-tree"
			return 2
		fi
		curd=$(dirname ${curd})
	done

	echo -n ""
	return 0
}

cdd ()
{
	([ -z "${1}" ] || [ "$1" == "-h" ]) && echo2 \
		"Usage: ${FUNCNAME[0]} <fname>\n" \
		"cd to parent-path of given file/directory up the dir-tree\n" \
		"<fname>: File/Directory name to cd to it's parent\n" \
		&& return 1

	abd ${1} && cd ${curd}
	return $?
}

_cdir ()
{
	cd ${1} || return 1

	local dirs=($(find . -maxdepth 1 -type d ! -name "." | sort))

	[ ${#dirs[@]} -eq 0 ] && return 0
	[ ${#dirs[@]} -eq 1 ] && cd ${dirs[0]} && return 0

	for (( i=0; i<${#dirs[@]}; i++ )); do
		echo "($((i+1))) ${dirs[$i]}"
	done

	read -p "Index of target directory: " i
	[[ $i ]] && [ $i -eq $i 2>/dev/null ] && \
		[ $i -gt 0 ] && [ $i -le ${#dirs[@]} ] && \
		cd ${dirs[$((i-1))]} || return 1

	return 0
}

# Usage: _CheckIfKnown <entry> <type> <array-name>
# <entry>   : string-with-no-space to find in array
# <type>    : type of array to use
#             n => simple; arr=(str1 str2 str3)
#             a => associative; declare -A arr=([a]="alpha" [b]="beta")
# <array-name> : just name of array passed as string
# e.g. invocation: _CheckIfKnown "x86" a "gKnownPlatforms"
# NOTE: If gKnownPlatforms,gSdkDirs,etc variables are not defined by
#       local-wee-env, _CheckIfKnown is either not called or it's
#       return value is discarded
# Return: 0 if entry is found in array, non-zero if not
_CheckIfKnown ()
{
	# UPDATE THIS FOR EVERY GENERIC ARRAY BEING CHECKED IN WEE
	declare -A key_name=( \
		["gSdkDirs"]="SDK Dir" \
		["gKnownPlatforms"]="Known Platform"
	)
	local entry="\<$1\>"
	local -n keys="$3"
	if [ -z "${!keys}" ]; then
		echo "Discarding ${key_name[$keys]}s as it's undefined for $WEE_ENV_ID!"
		return 2
	fi
	local vals="$3[@]"
	if [ "$2" == "a" ]; then
		[[ ${!keys[@]} =~ $entry ]] && return 0
	else
		[[ ${!vals} =~ $entry ]] && return 0
	fi
	echo "$entry: Entry not found in ${key_name[$keys]}s list"
	return 1
}

_sdk_ustr()
{
	# Called ONLY by sdk() to display help-string
	# as per environment!
	echo -e "Usage: sdk [opts]
	opts: Optional arguments only
	-h : display help message
	-s : just shows target directory path"
	if [ ${#gSdkDirs[@]} -ne 0 ]; then
		echo -e "\t-d <dir> : If specified and defined in gSdkDirs, uses <dir> as target path\n\t\t   dir options:"
		for key in "${!gSdkDirs[@]}"; do
			echo -e "\t\t   $key : ${gSdkDirs[$key]}"
		done
	fi
	if [ ${#gKnownPlatforms[@]} -ne 0 ]; then
		echo -e "\t-p <platform> : platform to choose
		\tIf given, goto sdk dir of a platform repo in ${wee}/../sdk/
		\tElse, goto sdk dir ONLY IF already in an SDK repo
		\tPlatform options: "
		for key in "${!gKnownPlatforms[@]}"; do
			echo -e "\t\t\t$key : ${gKnownPlatforms[$key]}"
		done
	fi
	echo -e "NOTE: Without any options, sdk goes to SDK's root directory\n"
}

# Invocation methods:
# 1. Mostly invoked with aliases
#    e.g. refer _wee-sample-local-no-run.sh
# 2. No aliases defined/needed for local-wee-env
#    e.g. sdk -d sdks
#    e.g. sdk -d sdkb -p imx6sll-evb
sdk ()
{
	local showPath=0
	local platform=""
	local dname=""
	while [[ $# -gt 0 ]]; do
		opt="$1"; shift
		case $opt in
			-h) echo2 "$(_sdk_ustr)"; return 0 ;;
			-s) showPath=1 ;;
			-d) _CheckIfKnown "$1" a gSdkDirs
				[ $? -ne 0 ] && echo2 "$(_sdk_ustr)" && return 1
				dname=$1; shift
				;;
			-p) _CheckIfKnown "$1" a gKnownPlatforms
				[ $? -ne 0 ] && echo2 "$(_sdk_ustr)" && return 1
				platform=$1; shift
				;;
			*) echo2 "Unknown option: $opt"
				echo2 "$(_sdk_ustr)"
				return -1
		esac
	done
	local localDir="${gSdkDirs[$dname]}"
	[ -z "${localDir}" ] && echo2 "$(_sdk_ustr)" && return -1
	if [ -z "$platform" ]; then
		local dirHandler=cdd ; local dirAction="cd"
		if [ $showPath -eq 1 ]; then
			dirHandler=abd ; dirAction="ls -d"
		fi
		$dirHandler .repo >/tmp/wee_sdk_err && ${dirAction} ${curd}/${localDir} && return 0
		$dirHandler .git >>/tmp/wee_sdk_err
		if [ $? -eq 0 ]; then
			local curdname="$(basename ${curd})"
			# User may try cd to root of target non-.repo SDK
			# Ensure we don't use dirname so we avoid /path-to/SDKdir/SDKdir situation
			if [ "${curdname}" == "$(echo ${localDir} | cut -d '/' -f1)"  ]; then
				# curd=/path-to/SDKdir
				# 1. if localDir=SDKdir,  make localDir=.
				# 2. if localDir=SDKdir/subdir/subdir1, make localDir="./subdir/subdir1"
				[ "${curdname}" == "${localDir}" ] && localDir="." || localDir="./${localDir#*/}"
			fi
			${dirAction} ${curd}/${localDir} && return 0
		fi
		# 1. Current PWD is not part of .repo or .git (OR)
		# 2. ${curd}/${localDir} doesn't exist
		# And no platform doesn't always mean local sdk-dir.
		# The sdk may need no platform specific info
		# So, just try ${wee}/../${WEE_ENV_ID}/
		# This is hard-coded approach if all cd attempts fail
		${dirAction} ${wee}/../${WEE_ENV_ID}/${localDir}
	else
		origd="${PWD}"
		_cdir ${wee}/../${WEE_ENV_ID}/$platform || return $?
		if [ $showPath -eq 1 ]; then
			ls -d ${PWD}/${localDir} && cd "${origd}"
		else
			cd ${localDir} && pwd
		fi
	fi
	return $?
}

weePlatform ()
{
	for i in ${!gKnownPlatforms[@]}; do
		printf "[%s] = %s\n" "$i" "${gKnownPlatforms[$i]}"
	done
}


###########################################################
##### Helpers for Gerrit actions on .repo or .git dir #####
###########################################################

# Internal function to validate gerrit remote info
# used in other gerrit helper functions
__ValidateGerritInfo ()
{
	___CheckVar () {
		[ -z "${2}" ] && echo2 "ERR: $1 undefined!" && return 1
		return 0
	}
	___CheckVar gDom "$gDom" && \
		___CheckVar gPort "$gPort" && \
		___CheckVar gSSHU "$gSSHU" && \
		___CheckVar gHTTPU "$gHTTPU" && \
		return 0 || return 1
}

__gerritIdSyntax="<gerrit>: Gerrit ID/link. Supported formats:
\t   https://review.jit.com/#/c/8206383/
\t   8206383
\t   I0c74ddefe0e86d8f7b5fC32f10c2f1165fa5347c8 (Change-Id)"

geri ()
{
	([ -z "${1}" ] || [ "$1" == "-h" ]) && echo2 \
		"\nUsage: ${FUNCNAME[0]} <gerrit> [opts]" \
		"\nGet complete info of a given gerrit in json format" \
		"\n\n${__gerritIdSyntax}" \
		"\n\nopts:" \
		"\n[-q]    : Optional, disables opening of output file" \
		"\n[-p]    : Optional, get only latest gerrit patchset info\n" \
		&& return 0

	__ValidateGerritInfo || return 1

	local quietMode=0
	local gerritInfoOpts="--all-approvals --files --comments --commit-message --dependencies --submit-records"

	local gid=$(basename "${1}")
	shift 1
	while [[ $# -gt 0 ]]; do
		opt="$1"
		case $opt in
			-q) quietMode=1; shift 1 ;;
			-p) gerritInfoOpts=""; shift 1 ;;
		esac
	done
	local jin=/tmp/$gid

	# Gerrit Query and adjust output json file
	ssh -p $gPort $(whoami)@${gDom} \
		gerrit query --current-patch-set \
		"${gerritInfoOpts}" \
		--format json change:$gid > $jin
	([ $? -ne 0 ] || \
		[ -n "$(grep "Error in operator change:$gid" $jin)" ]) && \
		echo2 "ERR: Gerrit query failed for $gid" && return 1
	sed -i '/^{"type":.*/d' $jin

	[ ! -s "$jin" ] && \
		echo2 "ERR: $jin is not valid json file" && \
		return 2

	echo "$jin"

	if [ $quietMode -eq 0 ]; then
		echo2 "Opening $jin .."
		sleep 1
		vi "$jin"
	fi

	return 0
}

# Internal function to find a project's sub-entry in manifest
_FindInManifest ()
{
	[ $# -ne 3 ] && echo "" && return
	python - ${1} ${2} ${3}<<END
import os, sys, xml.etree.ElementTree as xmlp
for c in xmlp.parse(sys.argv[1]).getroot().findall('project'):
	if c.attrib['name'] == sys.argv[2]:
		if sys.argv[3] == "path" : print (c.attrib[sys.argv[3]])
		else : print (os.path.basename(c.attrib[sys.argv[3]]))
		sys.exit (0)
END
}

gush ()
{
	([ "$1" == "-h" ] || [ "$(abd .git)" != "" ]) && \
		(echo2 "ERR: Not in .git repo\n" \
		"Usage: ${FUNCNAME[0]} [-a]\n" \
		"Does commit & push inside a .git repo.\n" \
		"Uses commit-stub (custom-commit-conf-cmd) if present\n" \
		"[-a] : Optional, used to amend last commit\n" \
		"[-s] : Optional, used to force sign commit\n" \
		&& return 0)

	local amend=""
	local sign=""
	while [[ $# -gt 0 ]]; do
		local opt=$1; shift
		case $opt in
			-a) amend="--amend" ;;
			-s) sign="-s" ;;
			*) echo2 "ERR: Unknown arg - $opt" && return -1 ;;
		esac
	done

	local gcommit="git commit"
	[ "$(type _gcommitstub 2>/dev/null)" ] && \
		gcommit=_gcommitstub
	eval "${gcommit} $sign $amend"
	local st=$?
	[ $st -ne 0 ] && \
		echo2 "ERR: Commit failed" && return $st

	local grem="$(git remote -v | grep push | awk '{print $1}')"
	[ -z "$grem" ] && \
		echo2 "ERR: ${PWD} remote unknown" && return 2
	local gpushcmd="git push --no-thin $grem HEAD:refs/for/"

	abd .repo
	if [ $? -eq 0 ]; then
		# In .repo dir, manifest file provides branch info
		local mf="${curd}/.repo/manifest.xml"
		[ ! -s "${mf}" ] && \
			echo2 "ERR: Repo-manifest not found" && return 1
		local frp=$(git remote -v | grep push | awk '{print $2}')
		local rp="${frp#*//}"; rp="${rp#*/}"; rp="${rp%'.git'}"
		local branch=$(_FindInManifest "${mf}" "${rp}" dest-branch)
		[ -z "${branch}" ] && \
			echo2 "ERR: ${rp} info not found in Repo-manifest" && return 3
	else
		# If git-clone of single project, use git branch info
		local branch="$(LESS=-eFRX \
			git branch | grep "\*" | awk '{print $2}')"
		([ -z "${branch}" ] || [[ "${branch}" == "("* ]]) && \
			echo2 "ERR: On this repo ${PWD}" && return 3
	fi
	eval "${gpushcmd}${branch}"

	return $?
}

grin ()
{
	([ -z "${1}" ] || [ "$1" == "-h" ]) && echo2 \
		"Usage: ${FUNCNAME[0]} [opts] <gerrit> [gerrit1 [gerrit2 ..]]" \
		"\n\nCherry-pick or Download patches from given gerrit ID/link(s)" \
		"\ninside a project within .repo sdk or single git-clone" \
		"\n\nFor .repo sdk: multiple projects cherry-pick their commits" \
		"\nFor .git clone: Only 1 project can cherry-pick its commits" \
		"\n\n${__gerritIdSyntax}" \
		"\nopts:" \
		"\n[-dl]    : Optional, just download patch files, don't cherry-pick\n" \
		&& return 0

	__ValidateGerritInfo || return 1

	local downloadPatchOnly=0
	local action="Cherry-pick"
	local gIDs=()
	while [[ $# -gt 0 ]]; do
		case $1 in
			-dl) downloadPatchOnly=1
				action="Download patch from"
				shift 1 ;;
			*) gIDs+=("$1"); shift 1 ;;
		esac
	done

	[ ${#gIDs[@]} -eq 0 ] && \
		echo2 "No gerrits specified!" && return 1

	local dotRepoFound=0
	cdd .repo && dotRepoFound=1

	local gid
	for gid in "${gIDs[@]}"; do
		gid=$(basename "${gid}")

		# Download Gerrit details
		local jin
		jin="$(geri $gid -q -p)"
		[ $? -ne 0 ] && \
			echo2 "Skipping ${FUNCNAME[0]} $gid.." && continue

		# Get Gerrit info from json file to local variables
		local ginfo="$(python ${wee}/jerrit.py $jin)"
		local project="$(echo "${ginfo}" | \
			grep '^project: ' | awk '{print $2}')"
		local ref="$(echo "${ginfo}" | \
			grep '^refs: ' | awk '{print $2}')"
		local status="$(echo "${ginfo}" | \
			grep '^status: ' | awk '{print $2}')"

		# If in a .repo project, cd to local project path
		if [ $dotRepoFound -eq 1 ]; then
			# In .repo dir, manifest file provides project's local path
			local mf="${curd}/.repo/manifest.xml"
			[ ! -s "${mf}" ] && \
				echo2 "ERR: Repo-manifest not found" && return 1
			local lpath=$(_FindInManifest "${mf}" "$project" path)
			[ -z "${lpath}" ] && \
				echo2 "ERR: $project info not found in Repo-manifest" && \
				return 3
			cd ${lpath} || return 3
		else
			[ ! -d ".git" ] && \
				echo2 "ERR: ${PWD} is not a git project" && return 3
			local gURL="$(git remote -v | grep push | awk '{print $2}')"
			if [[ "${gURL}" != *"${project}" ]]; then
				echo2 "ERR: Remote ${gURL} doesn't give '${project}'"
				return 3
			fi
		fi

		echo -e "\n---------------- ${action} $gid for $project"

		# If gerrit's merged, try git-checkout the commit
		if [ "$status" == "MERGED" ]; then
			local commit="$(echo "${ginfo}" | \
				grep '^commit: ' | awk '{print $2}')"
			[ -n "$(git log --pretty=oneline|grep ^$commit)" ] && \
				git format-patch $commit -1 && return 0
		fi

		#TODO  Not merged
		# Download patch with project-info,git-fetch,format-patch
		local ref="$(echo "${ginfo}" | \
			grep '^refs: ' | awk '{print $2}')"
		git fetch ${gSSHU}/$project ${ref} && \
			git format-patch -1 FETCH_HEAD || \
			return 4

		## Possible ways to apply gerrit patchsets with commits till END ##
		#repo download path/to/remote-proj 8206383/2
		#git fetch ssh://gerrit-review.kaum:29418/path/to/remote refs/changes/11/8206383/2 && git checkout FETCH_HEAD
		#git pull ssh://gerrit-review.kaum:29418/path/to/remote refs/changes/11/8206383/2
		#git fetch ssh://gerrit-review.kaum:29418/path/to/remote refs/changes/11/8206383/2 && git cherry-pick FETCH_HEAD
		#git fetch ssh://gerrit-review.kaum:29418/path/to/remote refs/changes/11/8206383/2 && git format-patch -1 --stdout FETCH_HEAD
		## END - Seleting cherry-pick as it helps best for conflict resolution and works on both .repo and individual git projects
		git fetch ${gSSHU}/${project} ${ref} && \
			git cherry-pick FETCH_HEAD || \
			(echo2 "ERR: ${project} failed to fetch from ${ref}" && \
			return 4)

		git format-patch -1 FETCH_HEAD || \
			(echo2 "ERR: ${project} format-patch failed from $gid" && \
			return 5)

		[ $dotRepoFound -eq 1 ] && cd ${origd}
		shift 1
	done

	return 0
}

rstat ()
{
	local force=0
	local showfile=1
	#Parse all input arguments
	while [[ $# -gt 0 ]]; do
		local opt="$1"
		case $opt in
			-f) force=1 ; shift 1 ;;
			-q) showfile=0 ; shift 1 ;; 
			*) [ "$opt" != "-h" ] && echo2 "Invalid argument: $opt"
				echo2 "Usage: ${FUNCNAME[0]} [opts]\n" \
					"opts:\n" \
					"  -h : Display usage string for rstat\n" \
					"  -f : Force check repo status\n" \
					"  -q : don't display files\n" \
					&& return 0 ;;
		esac
	done

	cdd ".repo" || return 1

	local patchDir="${curd}/rstat"

	# If rstat is not already present or if user forced, regenerate rstat
	if [ ! -d ${patchDir} ] || [ $force -eq 1 ]; then
		repo status > ${curd}/_frstat
		rm -rf ${patchDir}
		force=1
	fi

	# If not forced to generate new, just open existing rstat
	if [ $force -eq 0 ]; then
		cd ${origd}
		if [ $showfile -eq 1 ]; then
			[ ! -s ${patchDir}/frstat ] && \
				echo2 "No local modifications" || \
				vi ${patchDir}/frstat -O ${patchDir}/all.patch
		fi
		return 0
	fi

	# Backup current rstat for later reference if any repo's messed up
	[ -f ${patchDir} ] && mv ${patchDir} ${curd}/orstat

	mkdir -p ${patchDir} || return 2

	# Find modified files, their parent and git repo folders
	# to list down absolute paths w.r.t PWD to open directly without cd'ing
	local tmpPatch=/tmp/tmp.patch
	while read -r line <&3; do
		if [[ "${line}" == "project"* ]]; then
			local proj=$(echo "${line}" | awk '{print $2}')
			local projPatch=$(echo "${proj//\//$'_'}").patch
			cd ${proj}; git diff > ${tmpPatch}; cd - >/dev/null
			[ ! -s ${tmpPatch} ] && continue
			sed -i "1s#^#Patch for ${proj}\n#" ${tmpPatch}
			cp ${tmpPatch} ${patchDir}/${projPatch}
			cat ${tmpPatch} >> ${patchDir}/all.patch
			echo -e "\n------ ${proj} patch: ${projPatch}" >> ${patchDir}/frstat
			continue
		fi
		# It's a file/folder. Identify modified file(s) under $proj
		[ "$(echo ${line} | awk '{print $1}')" != "-m" ] && continue
		local file=$(echo ${line} | awk '{print $2}')
		echo "${proj}/${file}" >> ${patchDir}/frstat
	done 3<${curd}/_frstat
	rm -rf ${curd}/_frstat

	cd ${origd}
	if [ $showfile -eq 1 ]; then
		[ ! -s ${patchDir}/frstat ] && \
			echo2 "No local modifications done" || \
			vi ${patchDir}/frstat -O ${patchDir}/all.patch
	fi

	return 0
}

lopa ()
{
   local ustr="Usage: ${FUNCNAME[0]} [absolute-path-to-rstat-folder]
   Used to take an rstat/frstat to find .patch files and apply on current sdk.
   NOTE: 1. Must be run within a .repo/../<any-sub-dir>
         2. If path is not given, .repo/../rstat/ will be tried"

	[ "$1" == "-h" ] && echo2 "${ustr}" && return 0

   cdd ".repo" || (echo "${ustr}" && return 1)

	local patchDir=""
	[[ "${1}" == "/"* ]] && [ -s "${1}/frstat" ] && patchDir="${1}"
	[ ! -d "${patchDir}" ] && \
		[ -s "${curd}/rstat/frstat" ] && patchDir="${curd}/rstat"
   [ ! -d "${patchDir}" ] && \
		echo2 "ERR: No valid rstat provided/detected\n${ustr}" && \
		cd ${origd} && return 2

   while read -r line <&3; do
      [[ "${line}" != "--"* ]] && continue
      local dir="$(echo $line | awk '{print $2}')"
      cd ${dir}
      local patchf="$(echo $line | awk '{print $4}')"
      git apply ${patchDir}/"${patchf}" || \
         echo "Failed patching ${patchDir}/${patchf} on ${dir}"
   done 3<${patchDir}/frstat

   cd ${origd}

   return 0
}

grev ()
{
	[ "$1" == "-h" ] && echo2 \
		"${FUNCNAME[0]} [-a]"\
		"helps revert local modifications in a .repo sdk" \
		"\t-a : Optional, reverts all local without confirmation." \
		"       Without -a, ${FUNCNAME[0]} asks user-confirmation" \
		"       to revert each local modification" \
		"NOTE: Local commits, if present, will not be disturbed" \
		&& return 0

	cdd ".repo" || return 1

	local patchDir="${curd}/rstat"
	if [ ! -s ${patchDir}/frstat ]; then
		echo "Performing rstat first.." && rstat -f -q || return $?
	fi

	local interact=1
	[ "$1" == "-a" ] && interact=0

	while read -r line <&3; do
		[[ "${line}" != "--"* ]] && continue
		local dir="$(echo $line | awk '{print $2}')"
		cd ${dir}
		modified="$(git status | grep "modified:")"
		[ -n "${modified}" ] && echo "" && git checkout .
		cd -
	done 3<${patchDir}/frstat

	cd ${origd}

	return 0
}

# File containing Readable Gerrit Repo List (RGRL)
export RGRL=${wee}/ReadableGerritRepoList.txt

grls ()
{
	[ "$1" == "-h" ] && echo2 \
		"Usage: ${FUNCNAME[0]} [auto]\n" \
		"Load gerrit user-readable projects list to a file ${RGRL}" \
		"[auto] - Optional, is for automatic update intimation" \
		&& return 0

	# Since we source wee-local-* at EOF in weemain.sh, we wait
	# till gPort and gDom are exported by wee-local-* files
	while [ -z "${gPort}${gDom}" ]; do sleep 1; done
	[ -n "$(ps aux | grep ls-projects | grep -v grep)" ] && \
		echo "${RGRL} sync in progress" > /dev/stdout && return 0

	[ "$1" == "auto" ] && \
		echo "**** Auto-updating ReadableGerritRepoList"

	[ -f "${RGRL}" ] && chmod +w ${RGRL}
	ssh -p $gPort $(whoami)@${gDom} \
		gerrit ls-projects -d --format json --all > ${RGRL}
	local st=$?
	[ $st -ne 0 ] && \
		echo "ls-projects failed status=$st: $gPort $(whoami)@${gDom}"
	chmod -w ${RGRL}

	return 0
}

if [ ! -s ${RGRL} ]; then
	echo "No ${RGRL} found. Setting it up in background.."
	grls auto &
	echo -ne "Please run grls once a month or in need to update readable-projects-list"
	echo -ne " to be used by automated scripts to help with commit, push, fetch, etc"
fi


############### WEE Source local scripts ##################

# wee-local scripts are those that contain vars,funcs,etc
# for a specific dev-env (OpenWRT, Yocto, etc).
# wee-local scripts MUST BE NAMED wee-<name>-local.sh
# to be actually sourced per each shell session.
# Refer _wee-sample-local-no-run.sh
# e.g. wee-sdk-local.sh,wee-android-local.sh
# Now we either allow:
# 1. user to source weemain.sh with specific localwee as argument
#    # source ${wee}/weemain.sh <localwee>
#    # <localwee> being 'sdk' or 'android' from above example
# (OR)
# 2. sourcing an existing default-chosen localwee
#    # set default-localwee to avoid giving localwee everytime sourcing weemain.sh
# User decides approach 1 or 2 while sourcing from their bashrc/bash-profile scripts
localwee_sourced=false
if [ -n "$1" ]; then
	# User chose localwee to source. Do as user wishes
	[ -s "wee-$1-local.sh" ] && source "wee-$1-local.sh"
	f="wee-$1-local.sh"
elif [ ! -s ${wee}/default-localwee ]; then
	_localwees=($(find ${wee}/ -maxdepth 1 -type f -name "wee-*local.sh"))
	if [ ${#_localwees[@]} -eq 1 ]; then
		# Only 1 localwee, default to it
		echo ${_localwees[0]} > ${wee}/default-localwee
	elif [ ${#_localwees[@]} -gt 1 ]; then
		# More than 1 localwee, let user set default
		for (( i=0; i<${#_localwees[@]}; i++ )); do
			echo "($((i+1))) ${_localwees[$i]}"
			read -p "Set default localwee for future wee load: " i
			[[ $i ]] && [ $i -eq $i 2>/dev/null ] && \
				[ $i -gt 0 ] && [ $i -le ${#_localwees[@]} ] && \
				echo ${_localwees[$((i-1))]} > ${wee}/default-localwee
		done
	fi
	# default-localwee is set afresh, source that localwee
	f=$(cat ${wee}/default-localwee)
	[ -s "${f}" ] && source "${f}"
else
	# default-localwee is already set, source that localwee
	f=$(cat ${wee}/default-localwee)
	[ -s "${f}" ] && source "${f}"
	unset _localwees
fi
# Flag success/failure of sourcing of localwee above
if [ $? -eq 0 ]; then
	localwee_sourced=true
	wenv="$f"; wenv=${wenv%-*}; export WEE_ENV_ID=${wenv#*-}
	unset wenv
fi
unset f

#### sdk-env specific toolchain quick-access aliaes ####

# Quick SDK Gcc Toolchain Access Aliases QGTAA
if $localwee_sourced && [ -n "$(type _qgt 2>/dev/null)" ]; then
	alias qaddr2line="_qgt addr2line "${@}""
	alias qar="_qgt ar "${@}""
	alias qas="_qgt as "${@}""
	alias qc++="_qgt c++ "${@}""
	alias qc++filt="_qgt c++filt "${@}""
	alias qcpp="_qgt cpp "${@}""
	alias qelfedit="_qgt elfedit "${@}""
	alias qg++="_qgt g++ "${@}""
	alias qgcc="_qgt gcc "${@}""
	alias qgcov="_qgt gcov "${@}""
	alias qgcov-dump="_qgt gcov-dump "${@}""
	alias qgcov-tool="_qgt gcov-tool "${@}""
	alias qgdb="_qgt gdb "${@}""
	alias qgprof="_qgt gprof "${@}""
	alias qld="_qgt ld "${@}""
	alias qnm="_qgt nm "${@}""
	alias qobjcopy="_qgt objcopy "${@}""
	alias qobjdump="_qgt objdump "${@}""
	alias qranlib="_qgt ranlib "${@}""
	alias qreadelf="_qgt readelf "${@}""
	alias qsize="_qgt size "${@}""
	alias qstrings="_qgt strings "${@}""
	alias qstrip="_qgt strip "${@}""
fi

unset localwee_sourced
