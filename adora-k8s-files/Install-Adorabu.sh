# ==============================================================================
# Replace {SITE-FOLDERPTH} with your local path (e.g., /home/myusr/Projects/sitefiles)
# ==============================================================================
MOUNTPTH="{SITE-FOLDERPTH}/Adoraburu_RE/"
OBS_NAMESPACE="observatory"

kubectl delete -f pvolume.yaml --ignore-not-found
kubectl delete -f nginx-cong.yaml --ignore-not-found
kubectl delete -f apimanifest.yaml --ignore-not-found

minikube image build -t adorapiimg:1.8 "$MOUNTPTH/adoraburu_re-api/"

helm repo add prometh-comm-all https://prometheus-community.github.io/helm-charts 
helm repo add grafana https://grafana.github.io/helm-charts 
helm repo update 

helm upgrade --install prometheus prometh-comm-all/prometheus -f prometh-values.yaml -n $OBS_NAMESPACE --create-namespace
helm upgrade --install loki grafana/loki -f loki-values.yaml -n $OBS_NAMESPACE
helm upgrade --install promtail grafana/promtail -f promtail-values.yaml -n $OBS_NAMESPACE
helm upgrade --install grafana grafana/grafana -f grafana-values.yaml -n $OBS_NAMESPACE

minikube ssh "sudo mkdir -p /data/adoraburu-strg && sudo chmod 777 /data/adoraburu-strg"
minikube mount $MOUNTPTH:/data/adoraburu-strg --uid 101 --gid 101 &
sleep 2

kubectl apply -f pvolume.yaml
kubectl apply -f nginx-cong.yaml
kubectl apply -f apimanifest.yaml

minikube addons enable ingress

kubectl get pods,deployments
echo -e "\033[1;33mWait until all the pods are 'Ready'!\033[0m"

echo -e "\033[1;33mYou can access the site's folder in $MOUNTPTH\033[0m"

sudo sed -i '/adoraburu.utae/d' /etc/hosts
echo "$(minikube ip) adoraburu.utae" | sudo tee -a /etc/hosts

echo -e "\033[1;32mYou can access the site in your browser via the adoraburu.utae url!\033[0m"

