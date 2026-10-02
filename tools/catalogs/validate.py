#!/usr/bin/env python3
"""Offline validation for the draft content catalogs; standard library only."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
CATALOGS = ROOT / "data" / "catalogs"
FILES = ("biomes.json", "resources.json", "tech_tree.json", "governments.json", "magic_phenomena.json", "units.json", "buildings.json", "improvements.json", "event_templates.json")
EFFECT_OPS = {"AdjustResource", "AddTag", "RemoveTag", "CreateLedgerEntry", "SetTechnologyState", "EmitChronicle"}
PREDICATE_OPS = {"all", "any", "not", "compare", "has_tag", "exists"}
COMPARATORS = {"eq", "ne", "gt", "gte", "lt", "lte"}
YIELD_LIMITS = {"food": 4, "production": 3, "wealth": 3, "knowledge": 2, "culture": 2}


class ValidationError(Exception):
    pass


def reject_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValidationError(f"chave JSON duplicada: {key}")
        result[key] = value
    return result


def load(name: str) -> dict[str, Any]:
    try:
        with (CATALOGS / name).open(encoding="utf-8") as source:
            data = json.load(source, object_pairs_hook=reject_duplicates)
    except (OSError, json.JSONDecodeError, ValidationError) as exc:
        raise ValidationError(f"{name}: {exc}") from exc
    if not isinstance(data, dict):
        raise ValidationError(f"{name}: raiz deve ser objeto")
    return data


def integer(value: Any, path: str, low: int, high: int) -> None:
    if not isinstance(value, int) or isinstance(value, bool) or not low <= value <= high:
        raise ValidationError(f"{path}: esperado inteiro entre {low} e {high}")


def ids(items: list[dict[str, Any]], path: str) -> set[str]:
    found: set[str] = set()
    for index, item in enumerate(items):
        value = item.get("id")
        if not isinstance(value, str) or not value:
            raise ValidationError(f"{path}[{index}].id: ID ausente ou inválido")
        if value in found:
            raise ValidationError(f"{path}: ID duplicado: {value}")
        found.add(value)
    return found


def predicate(spec: Any, path: str) -> None:
    if not isinstance(spec, dict) or spec.get("op") not in PREDICATE_OPS:
        raise ValidationError(f"{path}: predicado fora da DSL")
    op = spec["op"]
    if op in {"all", "any"}:
        args = spec.get("args")
        if not isinstance(args, list) or not args:
            raise ValidationError(f"{path}: {op} exige args não vazios")
        for index, child in enumerate(args):
            predicate(child, f"{path}.args[{index}]")
    elif op == "not":
        predicate(spec.get("arg"), f"{path}.arg")
    elif op == "compare":
        if not isinstance(spec.get("fact"), str) or spec.get("comparator") not in COMPARATORS:
            raise ValidationError(f"{path}: comparação inválida")
        if not isinstance(spec.get("value"), (int, bool)):
            raise ValidationError(f"{path}: literal deve ser inteiro ou booleano")
    elif op == "has_tag":
        if not isinstance(spec.get("entity"), str) or not isinstance(spec.get("tag_id"), str):
            raise ValidationError(f"{path}: has_tag inválido")
    elif op == "exists":
        selector(spec.get("selector"), f"{path}.selector")


def selector(spec: Any, path: str) -> None:
    # JSON representa EntityKind(EntityKind) diretamente pelo tipo de entidade.
    allowed = {"self", "entity_kind", "related", "explicit_ids", "city", "civilization", "route", "tile"}
    if not isinstance(spec, dict) or spec.get("kind") not in allowed:
        raise ValidationError(f"{path}: seletor fora da DSL")


def effects(items: Any, path: str, resources: set[str]) -> None:
    if not isinstance(items, list) or len(items) > 64:
        raise ValidationError(f"{path}: lista de efeitos inválida")
    for index, effect in enumerate(items):
        at = f"{path}[{index}]"
        if not isinstance(effect, dict) or effect.get("op") not in EFFECT_OPS:
            raise ValidationError(f"{at}: operação de efeito fora da DSL")
        op = effect["op"]
        if op in {"AdjustResource", "AddTag", "RemoveTag", "SetTechnologyState"} and not isinstance(effect.get("target"), str):
            raise ValidationError(f"{at}: target obrigatório")
        if op == "AdjustResource":
            if effect.get("resource_id") not in resources:
                raise ValidationError(f"{at}: recurso inexistente")
            integer(effect.get("amount"), f"{at}.amount", -4, 4)
        if op in {"AddTag", "RemoveTag"} and not isinstance(effect.get("tag_id"), str):
            raise ValidationError(f"{at}: tag_id obrigatório")
        if op == "CreateLedgerEntry":
            if not all(isinstance(effect.get(key), str) for key in ("from", "to", "term_id")):
                raise ValidationError(f"{at}: ledger inválido")
            integer(effect.get("duration"), f"{at}.duration", 1, 12)
        if op == "EmitChronicle" and not isinstance(effect.get("template_id"), str):
            raise ValidationError(f"{at}: template_id obrigatório")


def resource_cost(cost: Any, path: str, resources: set[str], *, allow_zero: bool = False) -> None:
    if not isinstance(cost, dict) or not cost:
        raise ValidationError(f"{path}: custo ausente ou inválido")
    for resource_id, amount in cost.items():
        if resource_id not in resources:
            raise ValidationError(f"{path}: recurso inexistente {resource_id}")
        integer(amount, f"{path}.{resource_id}", 0 if allow_zero else 1, 100)


def technology_reference(value: Any, path: str, tech_ids: set[str]) -> None:
    if value not in tech_ids:
        raise ValidationError(f"{path}: tecnologia inexistente {value}")


def main() -> int:
    try:
        data = {name: load(name) for name in FILES}
        biomes = data["biomes.json"]["biomes"]
        resources_data = data["resources.json"]["resources"]
        biome_ids = ids(biomes, "biomes")
        resource_ids = ids(resources_data, "resources")
        if len(biomes) != 12:
            raise ValidationError("biomes: são exigidos exatamente 12 biomas")
        for biome in biomes:
            integer(biome.get("movement_cost"), f"biome {biome['id']}.movement_cost", 1, 4)
            yields = biome.get("yields")
            if not isinstance(yields, dict) or set(yields) != set(YIELD_LIMITS):
                raise ValidationError(f"biome {biome['id']}: rendimentos incompletos")
            for key, ceiling in YIELD_LIMITS.items():
                integer(yields[key], f"biome {biome['id']}.yields.{key}", 0, ceiling)
            for resource_id in biome.get("resource_tags", []):
                if resource_id not in resource_ids:
                    raise ValidationError(f"biome {biome['id']}: recurso inexistente {resource_id}")
        for resource in resources_data:
            for biome_id in resource.get("biomes", []):
                if biome_id not in biome_ids:
                    raise ValidationError(f"recurso {resource['id']}: bioma inexistente {biome_id}")
            category = resource.get("category")
            if category in {"finite", "strategic"}:
                integer(resource.get("stock_min"), f"recurso {resource['id']}.stock_min", 100, 400)
                integer(resource.get("stock_max"), f"recurso {resource['id']}.stock_max", 100, 400)
                integer(resource.get("extraction_per_worker"), f"recurso {resource['id']}.extraction_per_worker", 1, 2)
            if category in {"renewable", "luxury"}:
                integer(resource.get("yield_per_worker"), f"recurso {resource['id']}.yield_per_worker", 1, 2)
                integer(resource.get("regeneration_per_turn"), f"recurso {resource['id']}.regeneration_per_turn", 1, 1)
        techs = data["tech_tree.json"]["technologies"]
        tech_ids = ids(techs, "technologies")
        if not 15 <= len(techs) <= 25:
            raise ValidationError("technologies: esperado entre 15 e 25 entradas")
        for tech in techs:
            if tech.get("branch") not in {"sustenance", "organization", "circulation"}:
                raise ValidationError(f"tecnologia {tech['id']}: ramo inválido")
            integer(tech.get("cost"), f"tecnologia {tech['id']}.cost", 12, 30)
            for prerequisite in tech.get("prerequisites", []):
                if prerequisite not in tech_ids:
                    raise ValidationError(f"tecnologia {tech['id']}: pré-requisito inexistente")
            for biome_id in tech.get("requires_biomes", []):
                if biome_id not in biome_ids:
                    raise ValidationError(f"tecnologia {tech['id']}: bioma inexistente")
            practice = tech.get("practice", {})
            integer(practice.get("maintenance"), f"tecnologia {tech['id']}.practice.maintenance", 1, 3)
            effects(practice.get("effects"), f"tecnologia {tech['id']}.practice.effects", resource_ids)
        units = data["units.json"]["units"]
        ids(units, "units")
        if not 8 <= len(units) <= 14:
            raise ValidationError("unidades: esperado entre 8 e 14 entradas")
        roles = {"exploration", "defense", "attack", "settler", "worker", "trade"}
        for unit in units:
            if unit.get("role") not in roles:
                raise ValidationError(f"unidade {unit['id']}: função inválida")
            resource_cost(unit.get("cost"), f"unidade {unit['id']}.cost", resource_ids)
            resource_cost(unit.get("maintenance"), f"unidade {unit['id']}.maintenance", resource_ids)
            integer(unit.get("movement"), f"unidade {unit['id']}.movement", 1, 4)
            integer(unit.get("strength"), f"unidade {unit['id']}.strength", 0, 12)
            technology_reference(unit.get("requires_technology"), f"unidade {unit['id']}.requires_technology", tech_ids)
        if not roles <= {unit["role"] for unit in units}:
            raise ValidationError("unidades: faltam funções exigidas")
        buildings = data["buildings.json"]["buildings"]
        building_ids = ids(buildings, "buildings")
        if not 12 <= len(buildings) <= 20:
            raise ValidationError("edifícios: esperado entre 12 e 20 entradas")
        buildings_by_id = {building["id"]: building for building in buildings}
        function_tiers: set[tuple[str, int]] = set()
        for building in buildings:
            if not isinstance(building.get("function"), str) or not building["function"]:
                raise ValidationError(f"edifício {building['id']}: função ausente")
            integer(building.get("tier"), f"edifício {building['id']}.tier", 1, 4)
            pair = (building["function"], building["tier"])
            if pair in function_tiers:
                raise ValidationError(f"edifícios: nível duplicado para a função {building['function']}")
            function_tiers.add(pair)
            resource_cost(building.get("cost"), f"edifício {building['id']}.cost", resource_ids)
            resource_cost(building.get("maintenance"), f"edifício {building['id']}.maintenance", resource_ids)
            technology_reference(building.get("requires_technology"), f"edifício {building['id']}.requires_technology", tech_ids)
            effects(building.get("effects"), f"edifício {building['id']}.effects", resource_ids)
            previous = building.get("replaces")
            if previous is not None:
                if previous not in building_ids:
                    raise ValidationError(f"edifício {building['id']}: predecessor inexistente")
                predecessor = buildings_by_id[previous]
                if predecessor["function"] != building["function"] or predecessor["tier"] != building["tier"] - 1:
                    raise ValidationError(f"edifício {building['id']}: substituição deve ser o nível anterior da mesma função")
            elif building["tier"] != 1:
                raise ValidationError(f"edifício {building['id']}: nível superior exige replaces")
        improvements = data["improvements.json"]["improvements"]
        ids(improvements, "improvements")
        if not 8 <= len(improvements) <= 16:
            raise ValidationError("melhorias: esperado entre 8 e 16 entradas")
        resources_by_id = {resource["id"]: resource for resource in resources_data}
        for improvement in improvements:
            listed_biomes = improvement.get("biomes")
            listed_resources = improvement.get("resources")
            if not isinstance(listed_biomes, list) or not listed_biomes or not isinstance(listed_resources, list):
                raise ValidationError(f"melhoria {improvement['id']}: biomas ou recursos inválidos")
            for biome_id in listed_biomes:
                if biome_id not in biome_ids:
                    raise ValidationError(f"melhoria {improvement['id']}: bioma inexistente {biome_id}")
            for resource_id in listed_resources:
                if resource_id not in resource_ids:
                    raise ValidationError(f"melhoria {improvement['id']}: recurso inexistente {resource_id}")
                resource_biomes = set(resources_by_id[resource_id].get("biomes", []))
                if resource_biomes and not resource_biomes.intersection(listed_biomes):
                    raise ValidationError(f"melhoria {improvement['id']}: recurso incompatível com os biomas")
            resource_cost(improvement.get("cost"), f"melhoria {improvement['id']}.cost", resource_ids)
            resource_cost(improvement.get("maintenance"), f"melhoria {improvement['id']}.maintenance", resource_ids)
            technology_reference(improvement.get("requires_technology"), f"melhoria {improvement['id']}.requires_technology", tech_ids)
            effects(improvement.get("effects"), f"melhoria {improvement['id']}.effects", resource_ids)
        governments = data["governments.json"]["axes"]
        if len(governments) != 3 or any(len(axis.get("positions", [])) != 3 for axis in governments):
            raise ValidationError("governos: são exigidos 3 eixos com 3 posições")
        for axis in governments:
            for position in axis["positions"]:
                for policy in position.get("policies", []):
                    integer(policy.get("maintenance"), f"política {policy.get('id')}.maintenance", 1, 3)
                    integer(policy.get("stability_delta"), f"política {policy.get('id')}.stability_delta", -5, 5)
                    integer(policy.get("demand_satisfaction_delta"), f"política {policy.get('id')}.demand_satisfaction_delta", -10, 10)
                    effects(policy.get("effects"), f"política {policy.get('id')}.effects", resource_ids)
        phenomena = data["magic_phenomena.json"]["phenomena"]
        ids(phenomena, "phenomena")
        if not 3 <= len(phenomena) <= 5:
            raise ValidationError("fenômenos: esperado entre 3 e 5 entradas")
        for phenomenon in phenomena:
            predicate(phenomenon.get("entropy_trigger", {}).get("when"), f"fenômeno {phenomenon['id']}.when")
            for reaction in ("adopt", "prohibit", "regulate"):
                effects(phenomenon.get(reaction, {}).get("effects"), f"fenômeno {phenomenon['id']}.{reaction}.effects", resource_ids)
            for practice in phenomenon.get("practices", []):
                integer(practice.get("maintenance"), f"prática {practice.get('id')}.maintenance", 1, 3)
                effects(practice.get("effects"), f"prática {practice.get('id')}.effects", resource_ids)
        templates = data["event_templates.json"]["templates"]
        ids(templates, "event_templates")
        if len(templates) != 24 or sum(item.get("reference_event") is True for item in templates) != 5:
            raise ValidationError("eventos: exigidos 24 no total, incluindo 5 de referência")
        if sum(item.get("category") == "interference" for item in templates) < 3:
            raise ValidationError("eventos: exigidas 3 interferências na Entropia")
        required_categories = {"climate", "technology", "diplomacy", "revolt", "epidemic", "magic", "collapse", "renewal", "terrain"}
        if not required_categories <= {item.get("category") for item in templates}:
            raise ValidationError("eventos: faltam categorias exigidas")
        for template in templates:
            integer(template.get("tension_cost"), f"evento {template['id']}.tension_cost", 1, 4)
            integer(template.get("duration_turns"), f"evento {template['id']}.duration_turns", 1, 4)
            predicate(template.get("when"), f"evento {template['id']}.when")
            selector(template.get("targets"), f"evento {template['id']}.targets")
            effects(template.get("effects"), f"evento {template['id']}.effects", resource_ids)
            choices = template.get("choices")
            if not isinstance(choices, list) or len(choices) < 2:
                raise ValidationError(f"evento {template['id']}: resposta útil obrigatória exige ao menos 2 escolhas")
            ids(choices, f"evento {template['id']}.choices")
            for choice in choices:
                effects(choice.get("effects"), f"evento {template['id']}.escolha {choice.get('id')}", resource_ids)
            if template.get("category") == "interference":
                info = template.get("interference", {})
                if info.get("action") not in {"predict", "appease", "divert"} or "seeded" not in str(info.get("catastrophic_risk")):
                    raise ValidationError(f"evento {template['id']}: interferência sem risco seedado")
        print("catalog validation ok")
        return 0
    except (KeyError, TypeError, ValidationError) as exc:
        print(f"catalog validation failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
