#!/bin/bash
# Run as ./resources/icons/gen.sh !

cd resources/icons
export PATH=/home/$USER/Workspace/fontforgebuilds/ReleasePackage:$PATH

if [ -f `which run_fontforge` ]; then
    DIR="$(dirname "`which run_fontforge`")"
    FONTFORGE="$DIR/bin/fontforge.exe"
else
    FONTFORGE=`which fontforge`
fi

>&2 echo "Fontforge is $FONTFORGE"
$FONTFORGE -quiet -lang=py -script gen.py

if [ -z ../fonts ]; then
    mkdir ../fonts
fi

mv icons.ttf ../fonts
