import Table from "sap/m/Table";
import Formatter from "../model/Formatter";
import BaseTableController from "./BaseTable.controller";
import MyComponent from "de/regatta_hd/Component";
import JSONModel from "sap/ui/model/json/JSONModel";
import ViewSettingsFilterItem from "sap/m/ViewSettingsFilterItem";
import ViewSettingsItem from "sap/m/ViewSettingsItem";
import { ListBase$SelectEvent } from "sap/m/ListBase";
import Button, { Button$PressEvent } from "sap/m/Button";
import FilterOperator from "sap/ui/model/FilterOperator";
import Filter from "sap/ui/model/Filter";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import ViewSettingsDialog from "sap/m/ViewSettingsDialog";
import ListItemBase from "sap/m/ListItemBase";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import MessageToast from "sap/m/MessageToast";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class RacesTable extends BaseTableController {

  formatter: Formatter = Formatter;

  private racesModel: JSONModel;

  async onInit(): Promise<void> {
    super.init(super.getView()?.byId("racesTable") as Table, "race");

    super.getView()?.addStyleClass((super.getOwnerComponent() as MyComponent).getContentDensityClass());

    this.racesModel = await super.createJSONModel(`/api/regattas/${super.getRegattaId()}/races`, this.table);
    super.setViewModel(this.racesModel, "races");

    super.getRouter()?.getRoute("races")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadRacesModel(), this);

    const filters: any = (super.getComponentModel("filters") as JSONModel).getData();
    const viewSettingsDialog: ViewSettingsDialog = await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.RacesFilterDialog");

    if (filters.boatClasses) {
      const boatClassFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "boatClass", text: "{i18n>common.boatClass}" });
      filters.boatClasses.forEach((boatClass: any) => {
        boatClassFilter.addItem(new ViewSettingsItem({ text: boatClass.caption + " (" + boatClass.abbreviation + ")", key: "boatClass/id___EQ___" + boatClass.id }));
      });
      viewSettingsDialog.insertFilterItem(boatClassFilter, 0);
    }

    if (filters.ageClasses) {
      const ageClassFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "ageClass", text: "{i18n>common.ageClass}" });
      filters.ageClasses.forEach((ageClass: any) => {
        ageClassFilter.addItem(new ViewSettingsItem({ text: ageClass.caption + " " + ageClass.suffix, key: "ageClass/id___EQ___" + ageClass.id }));
      });
      viewSettingsDialog.insertFilterItem(ageClassFilter, 1);
    }

    if (filters.distances && filters.distances.length > 1) {
      const distancesFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "distance", text: "{i18n>common.distance}" });
      filters.distances.forEach((distance: any) => {
        distancesFilter.addItem(new ViewSettingsItem({ text: distance + "m", key: "distance___EQ___" + distance }));
      });
      viewSettingsDialog.insertFilterItem(distancesFilter, 3);
    }
  }

  onItemPress(event: ListBase$SelectEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext("races");
      const race: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      const index: number = this.table.indexOfItem(selectedItem);
      const count: number = this.table.getItems().length;
      // store navigation meta information in selected item
      race._nav = { isFirst: index == 0, isLast: index == count - 1 };

      this.onItemChanged(race);
      super.displayTarget("raceRegistrations");
    }
  }

  onNavBack(): void {
    super.navBack("startpage");
    // reduce table growing threshold to improve performance next time table is shown
    this.table.setGrowingThreshold(30);
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    if (query) {
      super.setSearchFilters(this.createSearchFilters(query));
    } else {
      super.setSearchFilters([]);
    }
    super.applyFilters();
  }

  async onFilterButtonPress(event: Button$PressEvent): Promise<void> {
    (await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.RacesFilterDialog")).open();
  }

  async onClearFilterPress(event: Button$PressEvent): Promise<void> {
    (await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.RacesFilterDialog")).clearFilters();
    super.clearFilters();
    super.applyFilters();
  }

  async onSortButtonPress(event: Button$PressEvent): Promise<void> {
    (await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.RacesSortDialog")).open();
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    const updated: boolean = await this.loadRacesModel();
    if (updated) {
      MessageToast.show(this.i18n("msg.dataUpdated"));
    }
    source.setEnabled(true);
  }

  onItemChanged(item: any): void {
    (super.getComponentModel("race") as JSONModel).setData(item);
    super.getEventBus()?.publish("race", "itemChanged", {});
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter("number", FilterOperator.Contains, query),
        new Filter("shortLabel", FilterOperator.Contains, query),
        new Filter("longLabel", FilterOperator.Contains, query),
        new Filter("comment", FilterOperator.Contains, query)
      ],
      and: false
    })]
  }

  private async loadRacesModel(): Promise<boolean> {
    return await super.updateJSONModel(this.racesModel, `/api/regattas/${super.getRegattaId()}/races`, this.table);
  }

}
