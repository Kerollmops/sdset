# run this script: `-e "benchname='intersection'"`

textcolor = "#1a1a1a"

set title benchname tc rgb textcolor

# stdout a png image
set terminal png size 720,480 enhanced

set auto x
set auto y

set ylabel "ns/iter" tc rgb textcolor

set key tc rgb textcolor

set style data histogram
set style histogram cluster gap 4
set style fill transparent solid 0.5

set boxwidth 3.5

set xtic scale 0
set xtics rotate by -45 offset 0,-0.5
set tics textcolor rgb textcolor

file = benchname.".data"

NC = system("awk 'NR==1{print NF}' ".file)

plot for [col=2:NC] file using col:xtic(1) ti col
