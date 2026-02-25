SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

cd $(SCRIPT_DIR)/shaders

rm *.spv
ls | xargs -I {} glslc - o $(cat {} | sed "s/\./_/g").spv
